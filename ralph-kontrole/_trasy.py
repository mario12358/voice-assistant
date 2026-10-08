"""Wspólne dla kontroli `intencja` i `red-team`: trasy HTTP z kodu, okno ich definicji
(gdzie stoją strażnicy) i mapa powierzchni ataku `ralph/BEZPIECZENSTWO.md`.

Ekstrakcja nie rozumie frameworka uwierzytelniania — i nie musi. Zwraca dla każdej trasy
**okno**: tekst dekoratorów i sygnatury (Python), argumentów wywołania przed handlerem (JS),
dekoratorów klasy i metody (Nest), plus definicję routera i middleware z prefiksem. Mapa mówi,
jaki tekst w tym oknie oznacza daną rolę („strażnik w kodzie"), więc sprawdzenie sprowadza się
do porównania tekstu. Słownik strażników pisze projekt, nie framework.

Pierwszy pomiar na Specky_app_v3: 233 dekoratory tras, z czego 117 ma ścieżkę w następnej
linii (`@router.get(\\n    "/x",`) — ekstrakcja linia po linii widziała połowę. Stąd sklejanie
nawiasów, zanim cokolwiek zostanie dopasowane.
"""
import os
import re
import subprocess

MAPA = 'ralph/BEZPIECZENSTWO.md'
PUSTE_DRZEWO = '4b825dc642cb6eb9a060e54bf8d69288fbee4904'
MAX_BAJTOW = 1_000_000
METODY_ZAPISU = {'POST', 'PUT', 'PATCH', 'DELETE'}

PLIK_TESTU = re.compile(r'(^|/)(tests?|__tests__|spec|e2e)/|(^|/)test_[^/]*$|[._-](test|spec)\.[^/]+$|_test\.[^/]+$'
                        r'|(^|/)conftest\.py$')
POMIJANE_KATALOGI = re.compile(r'(^|/)(node_modules|vendor|dist|build|out|target|\.venv|venv|__pycache__|'
                               r'\.next|\.nuxt|\.svelte-kit|coverage|site-packages|ralph-kontrole|\.claude)/')
KOD_PY = re.compile(r'\.py$')
KOD_JS = re.compile(r'\.(ts|tsx|js|jsx|mjs|cjs)$')
KOD = re.compile(r'\.(py|ts|tsx|js|jsx|mjs|cjs|go|java|kt|rb|php|cs|vue|svelte)$')
MIGRACJA = re.compile(r'(^|/)(migrations?|alembic|seeds?|db/migrate|supabase/migrations)/|\.sql$')

# --- Python: FastAPI, Flask, Starlette-podobne ---------------------------------------------------
PY_DEKORATOR_TRASY = re.compile(r'^\s*@(\w+)\.(get|post|put|patch|delete|head|options|api_route|route|websocket)\(')
PY_ROUTER = re.compile(r'^\s*(\w+)\s*(?::\s*[\w.\[\]]+)?\s*=\s*(?:fastapi\.|flask\.)?(APIRouter|Blueprint|FastAPI|Flask|Router)\(')
PY_DEF = re.compile(r'^\s*(?:async\s+)?def\s+\w+')
# --- JS / TS: Express, Hono, Fastify, Koa-router ------------------------------------------------
JS_TRASA = re.compile(r'\b(\w+)\.(get|post|put|patch|delete|all|head|options)\(\s*([\'"`])(/[^\'"`]*)\3')
JS_USE = re.compile(r'\b(\w+)\.use\(\s*(?:([\'"`])([^\'"`]*)\2\s*,)?')
# --- Nest ---------------------------------------------------------------------------------------
NEST_KONTROLER = re.compile(r'^\s*@Controller\(\s*(?:[\'"`]([^\'"`]*)[\'"`])?')
NEST_TRASA = re.compile(r'^\s*@(Get|Post|Put|Patch|Delete|All|Head|Options)\(\s*(?:[\'"`]([^\'"`]*)[\'"`])?')

# Plik JS jest serwerem, gdy importuje framework HTTP — inaczej `api.get('/x')` to klient (axios)
SERWER_JS = re.compile(r'(?:\bimport\b|\brequire\(|\bfrom\b)[^\n]*[\'"](?:npm:|jsr:|https://deno\.land/x/)?@?'
                       r'(?:express|hono|fastify|koa|koa/router|koa-router|nestjs/common|elysia|polka|restify|h3|'
                       r'oak|hono/[\w-]+)[@/\'"]')
KLIENCI_JS = {'axios', 'http', 'https', 'fetch', 'client', 'request', 'superagent', 'ky', 'got', 'this', 'map',
              'cache', 'store', 'params', 'headers', 'searchParams', 'url', 'cookies', 'localStorage',
              'sessionStorage', 'formData', 'res', 'req', 'c', 'ctx'}
LITERAL = re.compile(r'''("(?:[^"\\\n]|\\.)*"|'(?:[^'\\\n]|\\.)*'|`(?:[^`\\]|\\.)*`)''')


def git(root, *args, limit=60):
    try:
        r = subprocess.run(['git', '-C', root, *args], capture_output=True, text=True, timeout=limit)
    except (OSError, subprocess.TimeoutExpired):
        return None
    return r.stdout if r.returncode == 0 else None


def wczytaj(root, p):
    try:
        sc = os.path.join(root, p)
        if os.path.getsize(sc) > MAX_BAJTOW:
            return None
        with open(sc, 'rb') as f:
            dane = f.read()
    except OSError:
        return None
    if b'\0' in dane[:8192]:
        return None
    return dane.decode('utf-8', 'replace')


def tresc_z_rewizji(root, rew, p):
    """Treść pliku w rewizji; '' gdy pliku tam nie było (nowy plik = wszystko nowe)."""
    if rew == PUSTE_DRZEWO:
        return ''
    t = git(root, 'show', f'{rew}:{p}')
    return t if t is not None else ''


def punkt_odniesienia(root, punkt):
    """Rewizja, względem której coś jest „nowe". commit → HEAD; RALPH_OD (CI) bije resztę;
    faza/ci → ostatni tag ralph/faza-*, wydanie → ostatni v*; bez tagu — pierwszy commit."""
    if punkt == 'commit':
        h = (git(root, 'rev-parse', '--verify', '-q', 'HEAD') or '').strip()
        return h or PUSTE_DRZEWO
    od = os.environ.get('RALPH_OD', '').strip()
    if od and git(root, 'rev-parse', '--verify', '-q', od + '^{commit}'):
        return od
    wz = 'v[0-9]*' if punkt == 'wydanie' else 'ralph/faza-*'
    t = (git(root, 'describe', '--tags', '--abbrev=0', '--match', wz, 'HEAD') or '').strip()
    if t:
        return t
    pierwszy = (git(root, 'rev-list', '--max-parents=0', 'HEAD') or '').split()
    head = (git(root, 'rev-parse', 'HEAD') or '').strip()
    if not pierwszy or not head or head in pierwszy:
        return PUSTE_DRZEWO
    return pierwszy[-1]


def plik_kodu(p):
    return bool(KOD.search(p)) and not POMIJANE_KATALOGI.search(p) and not PLIK_TESTU.search(p)


# --- Ścieżki --------------------------------------------------------------------------------------

def kanon(sciezka):
    """`/projects/{project_id}/` → `/projects/{}`; `:id`, `<int:id>`, `[id]`, `${id}` → `{}`."""
    s = re.sub(r'\$\{[^}]*\}', '{}', (sciezka or '').strip())
    s = s.split('?')[0]
    seg = []
    for c in s.split('/'):
        if not c:
            continue
        if re.match(r'^(:\w+\??|\{[^}]*\}|<[^>]*>|\[[^\]]*\])$', c):
            seg.append('{}')
        else:
            seg.append(c)
    return '/' + '/'.join(seg)


def klucz(metoda, sciezka):
    return f'{metoda.upper()} {kanon(sciezka)}'


def pasuje_sciezka(znacznik, trasa):
    """Znacznik red teamu pasuje do trasy, gdy ścieżki są równe albo jedna jest sufiksem
    drugiej po pełnych segmentach — prefiks `/api` z `include_router` zna tylko test."""
    a = [x for x in kanon(znacznik).split('/') if x]
    b = [x for x in kanon(trasa).split('/') if x]
    if a == b:
        return True
    krotsza, dluzsza = sorted((a, b), key=len)
    return bool(krotsza) and dluzsza[-len(krotsza):] == krotsza


# --- Sklejanie nawiasów ---------------------------------------------------------------------------

def bez_literalow(t):
    return LITERAL.sub('""', t)


def sklej(wiersze, i, maks=40):
    """Od wiersza i do zamknięcia nawiasów otwartych w tym wierszu → (tekst, ostatni indeks)."""
    tekst, glebokosc, j = [], 0, i
    while j < len(wiersze) and j < i + maks:
        w = wiersze[j]
        tekst.append(w)
        czysty = bez_literalow(w.split('#')[0] if KOD_PY_KOMENTARZ.match(w) else w)
        glebokosc += czysty.count('(') + czysty.count('[') + czysty.count('{')
        glebokosc -= czysty.count(')') + czysty.count(']') + czysty.count('}')
        if glebokosc <= 0:
            break
        j += 1
    return '\n'.join(tekst), min(j, len(wiersze) - 1)


KOD_PY_KOMENTARZ = re.compile(r'^\s*[^\'"`]*#')


def pierwszy_literal(t):
    m = LITERAL.search(t)
    return m.group(0)[1:-1] if m else None


def wcieciem(w):
    return len(w) - len(w.lstrip())


# --- Python ---------------------------------------------------------------------------------------

def routery_py(wiersze):
    """{zmienna: (prefiks, tekst wywołania)} — `router = APIRouter(prefix=…, dependencies=[…])`."""
    out = {}
    for i, w in enumerate(wiersze):
        m = PY_ROUTER.match(w)
        if not m:
            continue
        tekst, _ = sklej(wiersze, i)
        p = re.search(r'(?:prefix|url_prefix)\s*=\s*[\'"]([^\'"]*)[\'"]', tekst)
        out[m.group(1)] = (p.group(1) if p else '', tekst)
    return out


def trasy_py(p, tekst):
    wiersze = tekst.split('\n')
    routery = routery_py(wiersze)
    out, i = [], 0
    while i < len(wiersze):
        w = wiersze[i]
        if not w.lstrip().startswith('@'):
            i += 1
            continue
        # blok dekoratorów + sygnatura funkcji
        start, dek, trasy, j = i, [], [], i
        while j < len(wiersze) and wiersze[j].lstrip().startswith('@'):
            t, k = sklej(wiersze, j)
            dek.append(t)
            m = PY_DEKORATOR_TRASY.match(wiersze[j])
            if m:
                trasy.append((m, t, j))
            j = k + 1
            while j < len(wiersze) and (not wiersze[j].strip() or wiersze[j].lstrip().startswith('#')):
                j += 1
        if not trasy or j >= len(wiersze) or not PY_DEF.match(wiersze[j]):
            i = max(j, i + 1)
            continue
        sygn, k = sklej(wiersze, j)
        while k + 1 < len(wiersze) and not re.search(r':\s*(#.*)?$', wiersze[k]):
            k += 1
            sygn += '\n' + wiersze[k]
        # koniec ciała: pierwsza niepusta linia z wcięciem ≤ dekoratora
        wc = wcieciem(wiersze[start])
        koniec = k + 1
        while koniec < len(wiersze):
            l2 = wiersze[koniec]
            if l2.strip() and not l2.lstrip().startswith('#') and wcieciem(l2) <= wc:
                break
            koniec += 1
        for m, t, nr in trasy:
            obiekt, rodzaj = m.group(1), m.group(2)
            arg = t[t.index('(') + 1:]
            if re.match(r'\s*[rbuf]?["\']', arg):
                sciezka = pierwszy_literal(arg) or ''
            else:
                mp = re.search(r'\b(?:path|rule)\s*=\s*[\'"]([^\'"]*)[\'"]', arg)
                sciezka = mp.group(1) if mp else ''
            prefiks, def_routera = routery.get(obiekt, ('', ''))
            if rodzaj in ('api_route', 'route'):
                mm = re.search(r'methods\s*=\s*[\[(]([^\])]*)', t)
                metody = re.findall(r'[\'"](\w+)[\'"]', mm.group(1)) if mm else ['GET']
            elif rodzaj == 'websocket':
                metody = ['WS']
            else:
                metody = [rodzaj]
            okno = '\n'.join(dek) + '\n' + sygn + '\n' + def_routera
            for met in metody:
                out.append({'plik': p, 'linia': nr + 1, 'metoda': met.upper(),
                            'sciezka': (prefiks.rstrip('/') + '/' + sciezka.lstrip('/')) if prefiks else (sciezka or '/'),
                            'okno': okno, 'od': start + 1, 'do': koniec, 'jezyk': 'py'})
        i = koniec
    return out


# --- JS / TS --------------------------------------------------------------------------------------

def okno_js(wiersze, i, kol):
    """Argumenty wywołania trasy do miejsca, gdzie zaczyna się handler (`=>`, `function`)."""
    tekst = wiersze[i][kol:]
    j = i
    while not re.search(r'=>|\bfunction\b|\)\s*;?\s*$', tekst) and j + 1 < len(wiersze) and j < i + 12:
        j += 1
        tekst += '\n' + wiersze[j]
    m = re.search(r'=>|\bfunction\b', tekst)
    return tekst[:m.start()] if m else tekst


FUNKCJA_JS = re.compile(r'(?:\bfunction\s+(\w+)\s*(?:<[^>]*>)?\s*\(|\b(?:const|let)\s+(\w+)\s*=\s*(?:async\s*)?\()'
                        r'([^)]*)\)', re.S)


def uzycia_js(tekst):
    """[(obiekt, prefiks albo None, tekst wywołania)] — `x.use(…)` w pliku."""
    wiersze = tekst.split('\n')
    out = []
    for i, w in enumerate(wiersze):
        for m in JS_USE.finditer(w):
            t, _ = sklej(wiersze, i, maks=8)
            out.append((m.group(1), m.group(3), t[t.find(m.group(0)):]))
    return out


_WOLAJACY = {}
_KONTEKST = {}
TWORZY_SERWER = re.compile(r'new\s+Hono\b|\bRouter\(|\bexpress\(\)|\bfastify\(|new\s+(?:Koa)?Router\b')


def wolajacy(root, p, nazwa):
    k = (root, nazwa)
    if k not in _WOLAJACY:
        pliki = (git(root, 'grep', '-l', '-w', '-e', nazwa, '--', '*.ts', '*.tsx', '*.js', '*.jsx', '*.mjs')
                 or '').splitlines()
        _WOLAJACY[k] = [x for x in pliki if not PLIK_TESTU.search(x) and not POMIJANE_KATALOGI.search(x)]
    return [x for x in _WOLAJACY[k] if x != p]


def pasujace_uzycia(uzycia, obiekt, prefiks):
    """Teksty `obiekt.use(…)` obowiązujące pod `prefiks` (globalne, `*`, albo prefiks pasujący)."""
    out = []
    for ob, pref, tx in uzycia:
        if ob != obiekt:
            continue
        czysty = (pref or '').rstrip('*').rstrip('/')
        if pref is None or not czysty or (prefiks or '/').startswith(czysty) or czysty.startswith(prefiks or '/'):
            out.append(tx)
    return out


def kontekst_montowania(root, p, tekst, glebokosc=0):
    """(prefiks, {obiekt albo '*': [tekst use]}) — skąd pod-aplikacja z pliku p dostaje prefiks
    i strażników. Dwa kształty, oba na altaforcie:
      - montowanie: `app.route("/doctor", createDoctorRoutes(…))` w main.ts → prefiks `/doctor`,
        strażnicy z `app.use` obowiązujący pod nim (i odziedziczeni przez main);
      - przekazanie obiektu: `trasyZespolow(routes, …)` w admin/index.tsx → prefiks i strażnicy
        pliku wołającego (`routes.use("*", requireRole("admin"))`).
    Bez prefiksu trasy różnych modułów mają ten sam klucz: na altaforcie `GET /` 10 razy,
    `GET /pacjenci/:id` lekarza i rejestracji jako jedna trasa. Do trzech poziomów, przez `git grep`."""
    k = (root, p)
    if not root or glebokosc > 3:
        return '', {}
    if k in _KONTEKST:
        return _KONTEKST[k] or ('', {})
    _KONTEKST[k] = None                           # w toku — cykl wywołań kończy się pustym kontekstem
    obiekty = {m.group(1) for m in JS_TRASA.finditer(tekst)} - KLIENCI_JS
    tworzy = bool(TWORZY_SERWER.search(tekst))
    wynik = ('', {})
    for m in FUNKCJA_JS.finditer(tekst):
        nazwa = m.group(1) or m.group(2)
        parametry = [x.split(':')[0].strip().lstrip('.') for x in m.group(3).split(',') if x.strip()]
        if not nazwa:
            continue
        przekazuje = obiekty & set(parametry)
        if not przekazuje and not tworzy:
            continue                              # tylko funkcje, które dostają albo budują obiekt tras
        for wp in wolajacy(root, p, nazwa):
            t = wczytaj(root, wp) or ''
            uz = uzycia_js(t)
            pref_w, dzied = kontekst_montowania(root, wp, t, glebokosc + 1)
            mm = re.search(r'\b(\w+)\.(?:route|use)\(\s*([\'"`])([^\'"`]*)\2\s*,\s*(?:await\s+)?'
                           + re.escape(nazwa) + r'\s*\(', t)
            if mm:
                lit = mm.group(3).rstrip('/')
                straz = pasujace_uzycia(uz, mm.group(1), lit) + dzied.get(mm.group(1), []) + dzied.get('*', [])
                wynik = ((pref_w.rstrip('/') + lit), {'*': straz})
                break
            mw = re.search(r'\b' + re.escape(nazwa) + r'\s*\(([^)]*)\)', t)
            if mw and przekazuje:
                argi = [x.strip() for x in mw.group(1).split(',')]
                straz = {}
                for idx, arg in enumerate(argi):
                    if idx < len(parametry) and parametry[idx] in obiekty and re.match(r'^\w+$', arg):
                        straz[parametry[idx]] = (pasujace_uzycia(uz, arg, '/') + dzied.get(arg, [])
                                                 + dzied.get('*', []))
                if straz:
                    wynik = (pref_w, straz)
                    break
        if wynik != ('', {}):
            break
    _KONTEKST[k] = wynik
    return wynik


def trasy_js(p, tekst, root=None):
    if not SERWER_JS.search(tekst):
        return []                                 # front / klient HTTP: `api.get('/users')` to wywołanie, nie trasa
    wiersze = tekst.split('\n')
    uzycia = uzycia_js(tekst)                     # (obiekt, prefiks albo None, tekst)
    prefiks_m, z_zewnatrz = kontekst_montowania(root, p, tekst)
    for ob, lista in z_zewnatrz.items():
        uzycia += [(ob, None, tx) for tx in lista]
    kontroler = None
    out, nest = [], []
    for i, w in enumerate(wiersze):
        if NEST_KONTROLER.match(w):
            kontroler = i
        for m in JS_TRASA.finditer(w):
            obiekt, metoda, sciezka = m.group(1), m.group(2), m.group(4)
            if obiekt in KLIENCI_JS:
                continue
            okno = okno_js(wiersze, i, m.start())
            for ob, pref, t in uzycia:
                if ob != obiekt and ob != '*':
                    continue
                if pref is None or sciezka.startswith(pref.rstrip('*').rstrip('/') or '/'):
                    okno += '\n' + t
            koniec = i + 1
            while koniec < len(wiersze) and not JS_TRASA.search(wiersze[koniec]) \
                    and not (wiersze[koniec].strip() and wcieciem(wiersze[koniec]) == 0
                             and not wiersze[koniec].lstrip().startswith(('}', ')'))):
                koniec += 1
            pelna = (prefiks_m + ('' if sciezka == '/' else sciezka)) if prefiks_m else sciezka
            out.append({'plik': p, 'linia': i + 1, 'metoda': metoda.upper(), 'sciezka': pelna or '/',
                        'okno': okno, 'od': i + 1, 'do': koniec, 'jezyk': 'js'})
        m = NEST_TRASA.match(w)
        if m and kontroler is not None:
            nest.append((i, m))
    if nest:
        # dekoratory klasy: ciągły blok wokół @Controller
        a = kontroler
        while a > 0 and wiersze[a - 1].lstrip().startswith('@'):
            a -= 1
        b = kontroler
        while b + 1 < len(wiersze) and not re.match(r'^\s*(export\s+)?(default\s+)?class\b', wiersze[b + 1]):
            b += 1
        klasa = '\n'.join(wiersze[a:b + 1])
        pm = NEST_KONTROLER.match(wiersze[kontroler])
        prefiks = '/' + pm.group(1).strip('/') if pm and pm.group(1) else ''
        for n, (i, m) in enumerate(nest):
            a = i
            while a > 0 and wiersze[a - 1].lstrip().startswith('@'):
                a -= 1
            b = i
            while b + 1 < len(wiersze) and wiersze[b + 1].lstrip().startswith('@'):
                b += 1
            koniec = nest[n + 1][0] if n + 1 < len(nest) else len(wiersze)
            sc = prefiks + ('/' + m.group(2).strip('/') if m.group(2) else '')
            out.append({'plik': p, 'linia': i + 1, 'metoda': m.group(1).upper(), 'sciezka': sc or '/',
                        'okno': klasa + '\n' + '\n'.join(wiersze[a:b + 2]), 'od': a + 1, 'do': koniec,
                        'jezyk': 'js'})
    return out


def trasy_pliku(p, tekst, root=None):
    """root (repozytorium) pozwala dociągnąć strażników z pliku wołającego (JS) — bez niego okno
    obejmuje tylko ten plik."""
    if not tekst:
        return []
    if KOD_PY.search(p):
        return trasy_py(p, tekst)
    if KOD_JS.search(p):
        return trasy_js(p, tekst, root)
    return []


# --- Kandydaci na strażników (tylko do szkieletu mapy) ---------------------------------------------

STRAZNIKOWY = re.compile(r'auth|user|role|member|admin|super|perm|login|guard|owner|access|scope|require|'
                         r'protect|ensure|verify|tenant|context|principal|identity|claims|current|jwt|session_user',
                         re.I)
NIE_STRAZNIK = re.compile(r'^(get_session|get_db|db|session|get_async_session|async_session|request|response|'
                          r'Depends|Security|Request|Response|async|await|function|return|const|let|var|new|true|'
                          r'false|null|undefined|UseGuards|Controller|SetMetadata)$|csrf|valid|schema|settings|config|service|repo|storage|limit|'
                          r'rate|cache|logger|json|body|query|param', re.I)


TOKEN_JS = re.compile(r'''"(?:[^"\\\n]|\\.)*"|'(?:[^'\\\n]|\\.)*'|`(?:[^`\\]|\\.)*`|[A-Za-z_$][\w$]*|=>|[(),.]''')
SEGMENT_JS = re.compile(r'\b\w+\.(?:get|post|put|patch|delete|all|head|options|use)\(')
PY_KANDYDAT = re.compile(r'(?:Depends|Security)\(\s*(\w+)(?:\(\s*[rbuf]?["\']([^"\']+)["\'])?')
DEKORATOR_KANDYDAT = re.compile(r'^\s*@(\w+)(?:\(\s*["\'`]([^"\'`]+)["\'`])?', re.M)


def argumenty_js(segment):
    """[(nazwa, literał albo None)] — argumenty NAJWYŻSZEGO poziomu wywołania `x.get(…)`/`x.use(…)`:
    referencja do middleware (`requireAuth`) albo fabryka (`requireRole("doctor")`). Głębiej leżące
    nazwy to argumenty strażników (`requireAuth(authorize)`) albo parametry handlera (`async (context)`)."""
    tok = TOKEN_JS.findall(segment)
    out, gl, i = [], 0, 0
    while i < len(tok):
        t = tok[i]
        if t == '(':
            gl += 1
        elif t == ')':
            gl -= 1
            if gl <= 0:
                break
        elif t == '=>' or (t in ('async', 'function') and gl == 1):
            break
        elif gl == 1 and re.match(r'^[A-Za-z_$]', t) and (i == 0 or tok[i - 1] != '.'):
            nast = tok[i + 1] if i + 1 < len(tok) else ''
            if nast == '(':
                lit = tok[i + 2] if i + 2 < len(tok) and tok[i + 2][:1] in '"\'`' else None
                out.append((t, lit[1:-1] if lit else None))
            elif nast != '.':
                out.append((t, None))
        i += 1
    return out


def kandydaci(trasa):
    """[(id roli, token strażnika)] z okna — heurystyka TYLKO do szkieletu mapy (sprawdzenie bierze
    strażników z mapy). Fabryka z literałem daje osobną rolę: `requireRole("doctor")` i
    `requireRole("nurse")` to dwie role, nie jedna `requireRole` — na altaforcie bez tego pięć ról
    personelu zlało się w jedną."""
    okno = trasa['okno']
    pary = []
    if trasa['jezyk'] == 'py':
        pary = PY_KANDYDAT.findall(okno) + DEKORATOR_KANDYDAT.findall(okno)
    elif '@Controller(' in okno:
        pary = DEKORATOR_KANDYDAT.findall(okno)
        pary += [(n, None) for n in re.findall(r'@UseGuards\(([^)]*)\)', okno) for n in re.findall(r'\w+', n)]
    else:
        for m in SEGMENT_JS.finditer(okno):
            pary += argumenty_js(okno[m.end() - 1:])
    out = []
    for nazwa, lit in pary:
        lit = lit or None
        if NIE_STRAZNIK.search(nazwa) or not STRAZNIKOWY.search(nazwa):
            continue
        if nazwa in ('get', 'post', 'put', 'patch', 'delete', 'route', 'api_route', 'use', 'app', 'router') \
                or re.search(r'(router|_app|_bp|blueprint)$', nazwa, re.I):
            continue                              # obiekt dekoratora (`@requirement_router.get`), nie strażnik
        para = (f'{nazwa}:{lit}', f'{nazwa}("{lit}")') if lit else (nazwa, nazwa)
        if para not in out:
            out.append(para)
    return out


def zawiera_straznika(okno, token):
    """Token z mapy w oknie trasy; cudzysłowy dowolne (`requireRole('doctor')` = `requireRole("doctor")`)."""
    norm = lambda t: re.sub(r'[\'`]', '"', t)
    tok, okno = norm(token), norm(okno)
    if tok in okno:
        return True
    # `requireRole("doctor")` pasuje też do `requireRole("doctor", "nurse")` — dostęp ma i lekarz
    return tok.endswith('")') and re.search(re.escape(tok[:-1]) + r'\s*,', okno) is not None


# --- Mapa -----------------------------------------------------------------------------------------

def wiersze_tabeli(blok):
    out = []
    for w in blok.split('\n'):
        w = w.strip()
        if not w.startswith('|') or re.match(r'^\|[\s:|-]+\|$', w):
            continue
        out.append([k.strip() for k in w.strip('|').split('|')])
    return out[1:] if out else []                 # pierwszy wiersz = nagłówek


def puste(v):
    return v.strip() in ('', '—', '-', '–', 'brak', 'nie')


def wczytaj_mape(root):
    """None gdy brak pliku. Inaczej {'role': {id: [tokeny]}, 'trasy': {klucz: {...}}, 'hosty', 'env',
    'baza', 'sesja'}."""
    t = wczytaj(root, MAPA)
    if t is None:
        return None
    t = re.sub(r'<!--.*?-->', '', t, flags=re.S)
    sekcje = {}
    for m in re.finditer(r'^## +(.+?)[ \t]*\n(.*?)(?=^## |\Z)', t, re.M | re.S):
        sekcje[m.group(1).strip().lower()] = m.group(2)
    s = re.search(r'^- \*\*Sesja\*\*:[ \t]*(.*?)[ \t]*$', t, re.M)
    mapa = {'role': {}, 'trasy': {}, 'hosty': set(), 'env': set(), 'baza': [], 'globalne': set(),
            'sesja': (s.group(1).strip().lower() if s else '?')}
    for w in wiersze_tabeli(sekcje.get('role', '')):
        if not w or not w[0]:
            continue
        rid = w[0].strip('`').strip()
        straz = w[2] if len(w) > 2 else ''
        tokeny = re.findall(r'`([^`]+)`', straz) or ([x.strip() for x in straz.split(',')] if not puste(straz) else [])
        tokeny = [x for x in tokeny if x and not puste(x)]
        if any(x.lower() in ('(globalny)', 'globalny', '(poza trasą)', '(poza trasa)') for x in tokeny):
            mapa['globalne'].add(rid)
        mapa['role'][rid] = [x for x in tokeny if not x.startswith('(')]
    for nr, w in enumerate(wiersze_tabeli(sekcje.get('trasy', ''))):
        if not w or not w[0]:
            continue
        m = re.match(r'^`?([A-Za-z]+)\s+(\S+?)`?$', w[0])
        if not m:
            continue
        mapa['trasy'][klucz(m.group(1), m.group(2))] = {
            'trasa': f'{m.group(1).upper()} {m.group(2)}',
            'role': w[1] if len(w) > 1 else '?',
            'wlasnosc': w[2] if len(w) > 2 else '?'}
    for w in wiersze_tabeli(sekcje.get('wyjścia', sekcje.get('wyjscia', ''))):
        if len(w) < 2:
            continue
        rodzaj, wart = w[0].strip('`').lower(), w[1].strip('`').strip()
        if rodzaj == 'host':
            mapa['hosty'].add(wart.lower())
        elif rodzaj == 'env':
            mapa['env'].add(wart)
        elif rodzaj == 'baza' and wart:
            mapa['baza'].append(re.sub(r'\s+', ' ', wart).lower())
    return mapa


def alternatywy(role):
    """`a + b, c` → [['a','b'], ['c']]; '?' → None (nieustalone)."""
    r = (role or '').strip()
    if r in ('', '?'):
        return None
    out, alts, gl, cur = [], [], 0, ''
    for ch in r:                                  # przecinek w nawiasie (`rola(OWNER, CLIENT)`) nie dzieli
        gl += ch == '('
        gl -= ch == ')'
        if ch == ',' and gl <= 0:
            alts.append(cur)
            cur = ''
        else:
            cur += ch
    alts.append(cur)
    for alt in alts:
        czl = [x.strip().strip('`') for x in alt.split('+') if x.strip()]
        czl = [re.sub(r'\(.*\)$', '', x).strip() for x in czl]
        if czl:
            out.append(czl)
    return out or None


def publiczna(role):
    alt = alternatywy(role)
    return bool(alt) and any(a == ['anonim'] for a in alt)
