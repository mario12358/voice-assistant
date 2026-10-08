#!/usr/bin/env python3
"""Kontrola `sast` — lista plików na stdin, znaleziska JSON w liniach na stdout.

Z semgrepem w PATH: `p/owasp-top-ten` + `p/security-audit` + pakiet językowy dla rozszerzeń
obecnych na wejściu. Bez niego (albo gdy nie zdąży / nie ma sieci po reguły) — tryb wbudowany:
kilkanaście wzorców per linia. Wzorce wbudowane działają też OBOK semgrepa: dokładają linie,
których on nie zgłosił (semgrep z pakietem OWASP nie zna np. `tempfile.mktemp`).

Semgrep wyłącza env `RALPH_SEMGREP=brak` (testy harnessu muszą dawać ten sam wynik z nim i bez).
"""
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys

PLIK_TESTU = re.compile(r'(^|/)(tests?|__tests__|spec|e2e)/|(^|/)test_[^/]*$|[._-](test|spec)\.[^/]+$|_test\.[^/]+$')
# Migracje i seedy budują SQL ze stałych (nazwy tabel w f-stringach) — reguła SQL by tam tylko szumiała
MIGRACJA = re.compile(r'(^|/)(migrations?|migrate|seeds?|alembic)/|\.sql$')
# Świadoma decyzja autora — ma być rzadka; preferowany znacznik to `ralph: osłabienie <id> — <powód>`,
# bo on jest rejestrem widocznym przy wydaniu, a `nosec` znika w kodzie
NOSEC = re.compile(r'(#|//|/\*)\s*(nosec|nosemgrep|noqa:\s*S\d*)\b')
KOMENTARZ = re.compile(r'^\s*(#|//|/\*|\*|<!--)')
IMPORT = re.compile(r'^\s*(import\s|from\s+[\w.]+\s+import\s|require\()')

JEZYKI = {'py': 'py', 'pyi': 'py',
          'js': 'js', 'jsx': 'js', 'ts': 'js', 'tsx': 'js', 'mjs': 'js', 'cjs': 'js', 'vue': 'js', 'svelte': 'js',
          'java': 'java', 'kt': 'java', 'scala': 'java', 'go': 'go', 'rb': 'rb', 'php': 'php', 'cs': 'cs',
          'html': 'szablon', 'htm': 'szablon', 'jinja': 'szablon', 'jinja2': 'szablon', 'j2': 'szablon',
          'ejs': 'szablon', 'hbs': 'szablon', 'njk': 'szablon', 'sql': 'sql'}
PAKIET_SEMGREP = {'py': 'p/python', 'js': 'p/javascript', 'go': 'p/golang'}
PAKIET_TS = ('ts', 'tsx')

# --- Rozpoznawanie argumentu ------------------------------------------------------------

LITERAL = re.compile(r'''^\s*[rRuUbB]{0,2}(?:"""|\'\'\'|"|')''')
FSTRING = re.compile(r'''^\s*(?:[rRbB]?[fF][rRbB]?)(?:"""|\'\'\'|"|')''')
FSTRING_W_SRODKU = re.compile(r'''(?<![\w.])[fF](?:"|')[^"']*\{''')
KONKATENACJA = re.compile(r'''["'`]\s*\+|\+\s*["'`]''')
FORMATOWANIE = re.compile(r'''["']\s*%\s*[\(\w]|\.format\(''')
SZABLON_JS = re.compile(r'`[^`]*\$\{')


LITERALY = re.compile(r'''"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'|`(?:\\.|[^`\\])*`''')


def szukaj(wzorzec, linia):
    """Pierwsze dopasowanie, które NIE zaczyna się w środku literału stringa — `r'--exec(=|$)'`
    w regexie sędziego to tekst, nie wywołanie (pierwszy przebieg po repo Ralpha)."""
    spany = None
    for m in wzorzec.finditer(linia):
        if spany is None:
            spany = [s.span() for s in LITERALY.finditer(linia)]
        if not any(a < m.start() < b for a, b in spany):
            return m
    return None


def argument(linia, wzorzec):
    """Tekst od pierwszego nawiasu wywołania pasującego do wzorca (bez nawiasu) albo None."""
    m = szukaj(wzorzec, linia)
    return linia[m.end():] if m else None


def sklejany(arg):
    """Czy argument jest SKLEJANY z danych: f-string z {}, konkatenacja, % / .format, template
    literal z ${}. Zmienna sama w sobie nie — może być przygotowanym zapytaniem."""
    s = arg.lstrip()
    if FSTRING.match(s):
        return '{' in s
    if s.startswith('`'):
        return '${' in s
    if LITERAL.match(s):
        return bool(KONKATENACJA.search(s) or FORMATOWANIE.search(s))
    return False


def dynamiczny(arg):
    """Jak `sklejany`, ale zmienna / wyrażenie zamiast literału też się liczy (komenda powłoki,
    eval: tam nie ma „przygotowanego" wariantu — literał to jedyny bezpieczny)."""
    s = arg.lstrip()
    if not s or s.startswith(')'):
        return False
    if FSTRING.match(s):
        return '{' in s
    if s.startswith('`'):
        return '${' in s
    if LITERAL.match(s) or s.startswith('['):
        return bool(KONKATENACJA.search(s) or FORMATOWANIE.search(s) or SZABLON_JS.search(s)
                    or FSTRING_W_SRODKU.search(s))
    return True


# `urllib.request.urlopen` to nie dane z żądania; `requests.get` ma `s` przed kropką, więc \b je odsiewa
ZRODLO_ZADANIA = re.compile(r'(?<!urllib\.)\brequest\.(?!urlopen|Request\b)|\breq\.(?:params|query|body|headers)\b'
                            r'|\bparams\[|\bquery\[|\bbody\[|flask\.request|\bargs\.get\(')

# --- Reguły blokujące ------------------------------------------------------------------

SQL_WYWOLANIE = re.compile(r'(?:\b|\.)(?:execute|executemany|executescript|raw|query)\(|(?<![\w.])text\(')
SQL_WYWOLANIE_JS = re.compile(r'(?:\b|\.)(?:execute|query|raw|unsafe|\$queryRawUnsafe|\$executeRawUnsafe)\(')
POWLOKA_PY = re.compile(r'\bsubprocess\.(?:run|call|Popen|check_output|check_call|getoutput|getstatusoutput)\(')
POWLOKA_OS = re.compile(r'\bos\.(?:system|popen)\(')
POWLOKA_JS = re.compile(r'(?<![\w])(?:exec|execSync)\(')
# `(?<!def )`: definicja metody `def exec(self, …)` to nie wywołanie (fałszywa blokada na Specky,
# scripts/prod_probe.py); `self.exec(` / `docker.exec(` odcina już `(?<![\w.])`
EVAL_PY = re.compile(r'(?<![\w.])(?<!def )(?:eval|exec)\(')
EVAL_JS = re.compile(r'(?<![\w.])eval\(')
EVAL_JS_ZAWSZE = re.compile(r'\bnew\s+Function\(|\b(?:setTimeout|setInterval)\(\s*["\']|\bvm\.runIn(?:New|This)Context\(')
DESERIALIZACJA = re.compile(r'\b(?:pickle|_pickle|cPickle)\.loads?\(|\bcPickle\b|\bmarshal\.loads?\(|\bshelve\.open\('
                            r'|\bjsonpickle\.decode\(|\bunserialize\(|\bObjectInputStream\b')
YAML_LOAD = re.compile(r'\byaml\.(?:load|load_all|unsafe_load)\(')
YAML_BEZPIECZNY = re.compile(r'SafeLoader|BaseLoader|safe_load')
SCIEZKA_SINK = re.compile(r'(?<![\w.])(?:open|Path)\(|\bsend_file\(|\bsendFile\(|\bFileResponse\(|\bos\.path\.join\('
                          r'|\breadFile(?:Sync)?\(|\bcreateReadStream\(|\bwriteFile(?:Sync)?\(')
SCIEZKA_SANITYZACJA = re.compile(r'secure_filename\(|\bbasename\(|safe_join\(|realpath\(|send_from_directory\(')
SCIEZKA_RESOLVE_Z_WERYFIKACJA = re.compile(r'path\.resolve\(.*(?:startsWith|startswith|relative_to|is_relative_to)\(')

# --- Reguły ostrzegające -----------------------------------------------------------------

XSS = re.compile(r'\.(?:innerHTML|outerHTML)\s*=(?!=)|\binsertAdjacentHTML\(|dangerouslySetInnerHTML|document\.write(?:ln)?\('
                 r'|\bv-html\s*=|\{@html\b|\|\s*safe\b|\bMarkup\(|\bmark_safe\(|\bbypassSecurityTrust\w+\(')
XSS_PUSTY = re.compile(r'''=\s*(["'`])\s*\1\s*;?\s*$''')
CORS_CREDENTIALS = re.compile(r'(?i)credentials\s*[:=]\s*true|supports_credentials\s*=\s*True|allow_credentials\s*=\s*True'
                              r'|Access-Control-Allow-Credentials["\']?\s*[:,=]\s*["\']?true')
CORS_DOWOLNY = re.compile(r'''(?i)origins?\s*[:=]\s*\[?\s*["']\*["']|origins?\s*[:=]\s*true\b'''
                          r'''|Access-Control-Allow-Origin["']?\s*[:,=]\s*["']\*|allow_origins?\s*=\s*\[?\s*["']\*''')
SLABY_HASH = re.compile(r'''\bhashlib\.(?:md5|sha1)\(|\bhashlib\.new\(\s*["'](?:md5|sha1)["']|\bcreateHash\(\s*["'](?:md5|sha1)["']'''
                        r'''|MessageDigest\.getInstance\(\s*["'](?:MD5|SHA-?1)["']|(?<![\w.])(?:md5|sha1)\(''')
HASLO_W_LINII = re.compile(r'(?i)password|passwd|has[łl]o|token|secret|\bpin\b')
LOSOWOSC = re.compile(r'(?<![\w.])random\.(?:random|randint|choice|choices|randrange|getrandbits|sample|uniform)\('
                      r'|Math\.random\(|(?<![\w.])(?:rand|mt_rand)\(')
SEKRET_W_LINII = re.compile(r'(?i)token|secret|session|\botp\b|nonce|password|reset|csrf|salt')
REDIRECT = re.compile(r'\b(?:redirect|RedirectResponse|HttpResponseRedirect)\(|\.(?:redirect)\(')
REDIRECT_ZRODLO = re.compile(r'(?<!urllib\.)\brequest\.(?:args|form|values|GET|POST|query_params|headers|referrer|params|get\()'
                             r'|\breq\.(?:query|params|body|headers)\b|\bargs\.get\(|(?<![/"\'\w])next\b|\breturn_?to\b'
                             r'|\breturn_?url\b|\bredirect_?(?:to|url|uri)\b')
REDIRECT_BEZPIECZNY = re.compile(r'url_has_allowed_host|is_safe_url|starts[wW]ith\(\s*["\']/|url_for\(|safe_redirect|validate_redirect|reverse\(')
STACKTRACE = re.compile(r'traceback\.format_exc(?:eption)?\(|\b(?:e|err|error|exc|exception|ex)\.stack\b')
ODPOWIEDZ = re.compile(r'\breturn\b|\bres\.(?:send|json|status|write|end)\(|\bjsonify\(|JSONResponse\(|HttpResponse\('
                       r'|\brender(?:_template)?\(|(?<![\w.])Response\(|\bctx\.body\b|\breply\.send\(')
JWT = re.compile(r'''verify_signature["']?\s*[:=]\s*False|algorithms\s*[:=]\s*\[[^\]]*["']none["']|\bjwt\.decode\(''')
JWT_VERIFY_FALSE = re.compile(r'\bverify\s*=\s*False')
XXE = re.compile(r'\betree\.(?:parse|fromstring|XML|iterparse|XMLParser)\(|\bminidom\.parse(?:String)?\(|\bxml\.sax\.'
                 r'|\bpulldom\.|\bexpatbuilder\b|\bresolve_entities\s*=\s*True|DocumentBuilderFactory\.newInstance\('
                 r'|\bSAXParserFactory\b|libxml_disable_entity_loader\(\s*false')
SSRF_SINK = re.compile(r'\brequests\.(?:get|post|put|patch|delete|head|request)\(|\bhttpx\.(?:get|post|put|patch|delete|request)\('
                       r'|(?<![\w.])fetch\(|\baxios(?:\.(?:get|post|put|patch|delete|request))?\(|\burlopen\(|\bgot(?:\.\w+)?\(')
TMP = re.compile(r'''\btempfile\.mktemp\(|(?<![\w.])open\(\s*["']/tmp/|\bos\.t(?:mp|emp)nam\(''')
LOG_SINK = re.compile(r'(?<![\w.])print\(|\b(?:log|logger|logging|console|LOG|LOGGER|_log|_logger|self\.log|self\.logger)'
                      r'\.(?:debug|info|warn|warning|error|critical|exception|log|fatal|trace|verbose)\('
                      r'|\b(?:log|fmt)\.Print(?:f|ln)?\(')
LOG_SEKRET_ID = re.compile(r'(?i)^(?:[a-z0-9]+_)*[a-z0-9]*(?:password|passwd|secret|api_?key|token|authorization'
                           r'|private_?key|client_?secret|credentials?)$')
# `key_marker(api_key)` (Specky, create-paddle-catalog.py) to też maskowanie — nazwa funkcji mówi, że
# wychodzi znacznik, nie wartość
LOG_MASKA = re.compile(r'\blen\(|\bbool\(|\bhash\(|mask|redact|marker|fingerprint|odcisk|skrot|ukryj|\*\*\*|\[:\d\]|\[-\d:\]|\.length\b', re.I)
def identyfikatory_poza_literalami(s):
    """Identyfikatory w kodzie; z literałów zostają tylko wstawki {…} / ${…} (f-string, template)."""
    def wstawki(m):
        return ' '.join(re.findall(r'\$?\{([^{}]*)\}', m.group(0)))
    return re.findall(r'[A-Za-z_][\w.]*', LITERALY.sub(wstawki, s))


def regula_sql(jez, linia):
    arg = argument(linia, SQL_WYWOLANIE_JS if jez == 'js' else SQL_WYWOLANIE)
    if arg is not None and sklejany(arg):
        return 'zapytanie SQL sklejane z danych (f-string / konkatenacja / % / .format / ${}) — użyj parametrów zapytania'


def regula_powloka(jez, linia):
    if jez == 'js':
        arg = argument(linia, POWLOKA_JS)
        # `re.exec(x)` też się tu łapie — wymagamy śladu sklejania, nie samej zmiennej
        if arg is not None and (SZABLON_JS.search(arg) or KONKATENACJA.search(arg)):
            return 'komenda powłoki sklejana z danych (exec/execSync z ${} lub +) — użyj execFile/spawn z tablicą argumentów'
        return None
    arg = argument(linia, POWLOKA_PY)
    if arg is not None and re.search(r'shell\s*=\s*True', linia) and dynamiczny(arg):
        return 'subprocess z shell=True i komendą budowaną z danych — przekaż listę argumentów bez shell=True'
    arg = argument(linia, POWLOKA_OS)
    if arg is not None and dynamiczny(arg):
        return 'os.system/os.popen z komendą budowaną z danych — użyj subprocess.run([...]) bez powłoki'


def regula_eval(jez, linia):
    if jez == 'js':
        if szukaj(EVAL_JS_ZAWSZE, linia):
            return 'wykonanie kodu ze stringa (new Function / setTimeout("…") / vm.runIn*Context) — nie da się tego bezpiecznie podać z danych'
        arg = argument(linia, EVAL_JS)
    else:
        arg = argument(linia, EVAL_PY)
    if arg is not None and dynamiczny(arg):
        return 'eval/exec na wyrażeniu, które nie jest literałem — zastąp parserem / słownikiem dozwolonych operacji'


def regula_deserializacja(jez, linia):
    if szukaj(DESERIALIZACJA, linia):
        return 'deserializacja formatu wykonującego kod (pickle / marshal / shelve / unserialize / ObjectInputStream) — użyj JSON'
    if szukaj(YAML_LOAD, linia) and not YAML_BEZPIECZNY.search(linia):
        return 'yaml.load bez SafeLoader — użyj yaml.safe_load'


def regula_sciezka(jez, linia):
    if szukaj(SCIEZKA_SINK, linia) and ZRODLO_ZADANIA.search(linia) \
            and not SCIEZKA_SANITYZACJA.search(linia) and not SCIEZKA_RESOLVE_Z_WERYFIKACJA.search(linia):
        return 'ścieżka pliku z danych żądania bez sanityzacji — secure_filename / basename / safe_join i sprawdzenie, że wynik leży w katalogu bazowym'


def regula_xss(jez, linia):
    m = szukaj(XSS, linia)
    if m and not XSS_PUSTY.search(linia):
        return f'HTML wstawiany bez escapowania ({m.group(0).strip()}) — textContent / escapowanie / sanitizer (DOMPurify)'


def regula_slaby_hash(jez, linia):
    if szukaj(SLABY_HASH, linia) and HASLO_W_LINII.search(linia):
        return 'md5/sha1 dla hasła lub tokenu — argon2 / bcrypt / scrypt (hasła), sha256+ (tokeny)'


def regula_losowosc(jez, linia):
    if szukaj(LOSOWOSC, linia) and SEKRET_W_LINII.search(linia):
        return 'losowość niekryptograficzna dla tokenu / sesji / OTP — secrets (Python) / crypto.randomBytes (Node)'


def regula_redirect(jez, linia):
    arg = argument(linia, REDIRECT)
    if arg is not None and REDIRECT_ZRODLO.search(arg) and not REDIRECT_BEZPIECZNY.search(linia):
        return 'przekierowanie na adres z żądania bez sprawdzenia hosta — dopuść tylko ścieżki względne albo listę domen'


def regula_stacktrace(jez, linia):
    if szukaj(STACKTRACE, linia) and ODPOWIEDZ.search(linia):
        return 'stack trace trafia do odpowiedzi HTTP — zostaw go w logu, klientowi zwróć identyfikator błędu'


def regula_jwt(jez, linia):
    m = szukaj(JWT, linia)
    if m:
        if m.group(0).startswith('jwt.decode'):
            if jez == 'js':
                return 'jwt.decode nie sprawdza podpisu — do uwierzytelnienia użyj jwt.verify'
            if 'algorithms' in linia:
                return None
            return 'jwt.decode bez algorithms=[…] — podaj listę algorytmów, inaczej „none" przechodzi'
        return 'weryfikacja JWT wyłączona (verify_signature=False / algorithms none)'
    if szukaj(JWT_VERIFY_FALSE, linia) and re.search(r'\bjwt\b|decode\(', linia):
        return 'weryfikacja podpisu JWT wyłączona (verify=False)'


def regula_xxe(jez, linia):
    if szukaj(XXE, linia):
        return 'parser XML bez ochrony przed encjami zewnętrznymi (XXE) — defusedxml / resolve_entities=False / wyłączone DTD'


def regula_ssrf(jez, linia):
    arg = argument(linia, SSRF_SINK)
    if arg is not None and ZRODLO_ZADANIA.search(arg):
        return 'żądanie HTTP na adres z danych żądania (SSRF) — lista dozwolonych hostów, bez adresów prywatnych'


def regula_tmp(jez, linia):
    if szukaj(TMP, linia):
        return 'niebezpieczny plik tymczasowy (mktemp / stała ścieżka w /tmp) — tempfile.NamedTemporaryFile / mkstemp'


def regula_logowanie(jez, linia):
    arg = argument(linia, LOG_SINK)
    if arg is None or LOG_MASKA.search(arg):
        return None
    for ident in identyfikatory_poza_literalami(arg):
        if LOG_SEKRET_ID.match(ident.rsplit('.', 1)[-1]):
            return f'sekret w logu (`{ident[:40]}`) — loguj fakt, nie wartość'


# (reguła, waga, funkcja) — kolejność = kolejność sprawdzania; jedna linia może trafić w kilka reguł
REGULY = [
    ('sql-konkatenacja', 'blokuje', regula_sql),
    ('komenda-powloki', 'blokuje', regula_powloka),
    ('eval-exec', 'blokuje', regula_eval),
    ('deserializacja', 'blokuje', regula_deserializacja),
    ('path-traversal', 'blokuje', regula_sciezka),
    ('xss-innerhtml', 'ostrzega', regula_xss),
    ('slaby-hash', 'ostrzega', regula_slaby_hash),
    ('losowosc-niekryptograficzna', 'ostrzega', regula_losowosc),
    ('redirect-otwarty', 'ostrzega', regula_redirect),
    ('debug-stacktrace-w-odpowiedzi', 'ostrzega', regula_stacktrace),
    ('jwt-bez-weryfikacji', 'ostrzega', regula_jwt),
    ('xxe', 'ostrzega', regula_xxe),
    ('ssrf', 'ostrzega', regula_ssrf),
    ('tmp-niebezpieczny', 'ostrzega', regula_tmp),
    ('logowanie-sekretu', 'ostrzega', regula_logowanie),
]
# Reguły, które w danym języku nie mają sensu (eval w szablonie HTML to tekst, nie kod)
TYLKO_KOD = {'sql-konkatenacja', 'komenda-powloki', 'eval-exec', 'deserializacja', 'path-traversal',
             'slaby-hash', 'losowosc-niekryptograficzna', 'redirect-otwarty', 'debug-stacktrace-w-odpowiedzi',
             'jwt-bez-weryfikacji', 'xxe', 'ssrf', 'tmp-niebezpieczny', 'logowanie-sekretu'}


def odcisk(t):
    return hashlib.sha256(t.encode('utf-8', 'replace')).hexdigest()[:16]


def wypisz(**z):
    print(json.dumps(z, ensure_ascii=False))


def czy_binarny(dane):
    return b'\0' in dane[:8192]


def jezyk(p):
    return JEZYKI.get(p.rsplit('.', 1)[-1].lower() if '.' in os.path.basename(p) else '')


def wczytaj(root, p):
    try:
        with open(os.path.join(root, p), 'rb') as f:
            dane = f.read()
    except OSError:
        return None
    if czy_binarny(dane):
        return None
    return dane.decode('utf-8', 'replace').splitlines()


def wbudowany(root, pliki, pominiete_linie):
    """Wzorce per linia. `pominiete_linie` = {(plik, nr)} zgłoszone już przez semgrepa."""
    for p in pliki:
        jez = jezyk(p)
        if jez is None or jez == 'sql':
            continue
        wiersze = wczytaj(root, p)
        if wiersze is None:
            continue
        tresc = '\n'.join(wiersze)
        bez_sql = bool(MIGRACJA.search(p))
        bez_xxe = 'defusedxml' in tresc     # plik już używa bezpiecznego parsera — reszta to szum
        trafienia = {}      # regula → [(nr, linia, opis)]
        for nr, linia in enumerate(wiersze, 1):
            if len(linia) > 4000 or KOMENTARZ.match(linia) or IMPORT.match(linia) or NOSEC.search(linia):
                continue
            if (p, nr) in pominiete_linie:
                continue
            for regula, waga, fn in REGULY:
                if jez == 'szablon' and regula in TYLKO_KOD:
                    continue
                if (regula == 'sql-konkatenacja' and bez_sql) or (regula == 'xxe' and bez_xxe):
                    continue
                opis = fn(jez, linia)
                # `jwt.decode(` z argumentami w kolejnych liniach: `algorithms=[…]` stoi niżej
                # (backend/app/auth/oauth_clients.py na Specky) — patrzymy w okno wywołania
                if opis and regula == 'jwt-bez-weryfikacji' and opis.startswith('jwt.decode bez') \
                        and any('algorithms' in w for w in wiersze[nr:nr + 5]):
                    opis = None
                if opis:
                    trafienia.setdefault(regula, []).append((nr, linia, opis))
            # Jedyna reguła z kontekstem: origin i credentials zwykle stoją w sąsiednich liniach
            if szukaj(CORS_CREDENTIALS, linia):
                okno = wiersze[max(0, nr - 4):nr + 3]
                if any(CORS_DOWOLNY.search(w) for w in okno):
                    trafienia.setdefault('cors-dowolny-z-credentials', []).append(
                        (nr, linia, 'CORS: dowolne pochodzenie razem z credentials — każda strona może '
                                    'wołać API z ciasteczkami użytkownika; podaj konkretne domeny'))
        # Reguła raz na plik z licznikiem (jak w oslabienia). Odcisk liczony ze WSZYSTKICH trafionych
        # linii, nie z pierwszej: wyjątek po odcisku ma zdjąć dokładnie te linie, które człowiek widział —
        # nowe wstrzyknięcie dopisane do pliku z wyjątkiem musi zablokować od nowa
        wagi = {r: w for r, w, _ in REGULY}
        for regula, lista in trafienia.items():
            nr, linia, opis = lista[0]
            dalej = f' (+{len(lista) - 1} dalej w pliku)' if len(lista) > 1 else ''
            linie = sorted({l.strip() for _, l, _ in lista})
            wypisz(plik=p, linia=nr, regula=regula, waga=wagi.get(regula, 'ostrzega'),
                   opis=f'{opis}{dalej}', odcisk_tresci=odcisk('\n'.join(linie)))


# --- Semgrep ----------------------------------------------------------------------------

BLOKUJACE_ID = ('injection', 'sqli', 'sql-injection', 'command', 'subprocess', 'eval', 'exec', 'deserial',
                'pickle', 'yaml-load', 'path-traversal', 'tainted-path', 'ssrf')


def semgrep_dostepny():
    return bool(shutil.which('semgrep')) and os.environ.get('RALPH_SEMGREP', '').lower() != 'brak'


def z_semgrepa(root, pliki, punkt):
    """Zwraca ({(plik, nr)} zgłoszonych linii, None) albo (set(), powód), gdy semgrep nie dał się użyć."""
    pakiety = ['p/owasp-top-ten', 'p/security-audit']
    rozszerzenia = {p.rsplit('.', 1)[-1].lower() for p in pliki if '.' in os.path.basename(p)}
    for jez, pakiet in PAKIET_SEMGREP.items():
        if any(JEZYKI.get(r) == jez for r in rozszerzenia):
            pakiety.append(pakiet)
    if rozszerzenia & set(PAKIET_TS):
        pakiety.append('p/typescript')
    cmd = ['semgrep', 'scan']
    for pk in pakiety:
        cmd += ['--config', pk]
    cmd += ['--json', '--quiet', '--metrics=off', '--timeout', '10', *pliki]
    limit = 15 if punkt == 'commit' else 240
    try:
        r = subprocess.run(cmd, cwd=root, capture_output=True, text=True, timeout=limit)
    except subprocess.TimeoutExpired:
        return set(), f'semgrep nie zmieścił się w {limit} s'
    except OSError as e:
        return set(), f'semgrep nie uruchomił się ({e})'
    if r.returncode not in (0, 1):
        powod = (r.stderr.strip().splitlines() or ['?'])[-1][:120]
        return set(), f'semgrep zakończył się kodem {r.returncode} ({powod}) — brak sieci po reguły p/…?'
    try:
        dane = json.loads(r.stdout)
        wyniki = dane['results']
    except (ValueError, KeyError, TypeError):
        return set(), 'semgrep zwrócił nieczytelny wynik'
    zgloszone = set()
    for w in wyniki:
        plik = os.path.relpath(os.path.join(root, w.get('path', '')), root)
        nr = (w.get('start') or {}).get('line')
        check_id = w.get('check_id', '')
        extra = w.get('extra') or {}
        severity = (extra.get('severity') or '').upper()
        regula = check_id.rsplit('.', 1)[-1] or 'semgrep'
        # Migracje budują SQL ze stałych — ta sama zasada co dla wzorca wbudowanego
        # (`sqlalchemy.text` w alembic/versions/ dawał na Specky trzy ostrzeżenia bez treści)
        if MIGRACJA.search(plik) and 'sql' in check_id.lower():
            zgloszone.add((plik, nr))
            continue
        blokuje = severity == 'ERROR' and any(k in check_id.lower() for k in BLOKUJACE_ID)
        kod = ((extra.get('lines') or '').strip().splitlines() or [''])[0].strip()
        wiadomosc = ' '.join((extra.get('message') or regula).split())[:200]
        fragment = f' — `{kod[:80]}`' if kod else ''
        wypisz(plik=plik, linia=nr, regula=regula, waga='blokuje' if blokuje else 'ostrzega',
               opis=f'{wiadomosc}{fragment}', odcisk_tresci=odcisk(kod))
        zgloszone.add((plik, nr))
    return zgloszone, None


def main():
    if '--sprawdz' in sys.argv:
        if semgrep_dostepny():
            r = subprocess.run(['semgrep', '--version'], capture_output=True, text=True)
            print(f'ok: semgrep {r.stdout.strip().splitlines()[-1] if r.stdout.strip() else "?"} '
                  f'(p/owasp-top-ten, p/security-audit + pakiet językowy)')
        else:
            print('uwaga: sast w trybie wbudowanym (kilkanaście wzorców) — zainstaluj semgrep '
                  '(pip install semgrep / brew install semgrep) dla pełnych reguł OWASP')
        return 0
    root = os.environ.get('RALPH_ROOT') or os.getcwd()
    punkt = os.environ.get('RALPH_PUNKT', 'commit')
    pliki = [l.strip() for l in sys.stdin.read().splitlines() if l.strip()]
    # Pliki testów w całości poza kontrolą: test legalnie robi eval, sklejony SQL i pickle
    pliki = [p for p in pliki if not PLIK_TESTU.search(p) and jezyk(p)]
    if not pliki:
        return 0
    zgloszone = set()
    if semgrep_dostepny():
        zgloszone, powod = z_semgrepa(root, pliki, punkt)
        if powod:
            wypisz(plik=pliki[0], linia=None, regula='nie-sprawdzono', waga='ostrzega', odcisk_pliku=False,
                   opis=f'{powod} — sprawdzono tylko wzorcami wbudowanymi', odcisk_tresci='semgrep')
    wbudowany(root, pliki, zgloszone)
    return 0


if __name__ == '__main__':
    sys.exit(main())
