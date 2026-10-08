#!/usr/bin/env python3
"""Kontrola `martwe-wejscie` — lista plików na stdin (korpus odwołań), znaleziska JSON w liniach.

Sprawdza TYLKO symbole nowe od punktu odniesienia (tag fazy / wydania / RALPH_OD / pierwszy
commit): nowa funkcja, klasa, eksport, trasa HTTP albo komponent, do których poza własnym
testem nic się nie odwołuje. Istniejący martwy kod to osobny problem — pełna lista przy każdej
fazie byłaby szumem. Nigdy nie blokuje: martwy kod nie psuje działania, a precyzja tej
kontroli nie jest jeszcze zmierzona.
"""
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys

PLIK_TESTU = re.compile(r'(^|/)(tests?|__tests__|spec|e2e)/|(^|/)test_[^/]*$|[._-](test|spec)\.[^/]+$|_test\.[^/]+$')
STORIES = re.compile(r'\.stories\.[^/]+$')
# Korpus odwołań: kod + pliki, w których rejestruje się punkty wejścia (entry points w pyproject,
# komendy w Makefile / Dockerfile / skryptach, nazwy klas w konfiguracji). Bez .md — wzmianka
# w dokumentacji to nie wywołanie.
KOD = re.compile(r'\.(py|pyi|js|jsx|ts|tsx|mjs|cjs|vue|svelte|go|html|htm|toml|cfg|ini|ya?ml|json|sh|bash|'
                 r'tf|proto|graphql|sql)$|(^|/)(Makefile|Dockerfile|Procfile|Justfile)$')
# Pliki frontu / klienta / dokumentacji API — tylko tu szukamy odwołań do tras HTTP
FRONT = re.compile(r'\.(ts|tsx|js|jsx|mjs|vue|svelte|html|htm|md|ya?ml|json)$')
# Repo „ma front" = jest czym wywołać trasę. Sam backend (klient w innym repo) → tras nie sprawdzamy
MA_FRONT = re.compile(r'\.(tsx|jsx|vue|svelte|html|htm)$|(^|/)(openapi|swagger)[^/]*$|'
                      r'(^|/)(frontend|client|clients|web|webapp|ui|www|static|public)/.*\.(ts|js|mjs)$')
POMIJANE_KATALOGI = re.compile(r'(^|/)(node_modules|vendor|dist|build|out|target|\.venv|venv|__pycache__|'
                               r'\.next|\.nuxt|\.svelte-kit|coverage|site-packages)/')
POMIJANE_PLIKI = re.compile(r'\.min\.(js|css)$|\.map$|\.lock$|(^|/)(package-lock\.json|yarn\.lock|pnpm-lock\.yaml|'
                            r'poetry\.lock|Cargo\.lock|go\.sum|deno\.lock|composer\.lock|Gemfile\.lock)$')
MAX_BAJTOW = 1_000_000
MAX_ZNALEZISK = 30
LIMIT_NARZEDZIA = 60
PUSTE_DRZEWO = '4b825dc642cb6eb9a060e54bf8d69288fbee4904'

# --- Python ---------------------------------------------------------------------------
PY_DEF = re.compile(r'^(?:async\s+)?def\s+(\w+)\s*\(|^class\s+(\w+)\b')
# Pliki, w których framework rejestruje handlery przez konwencję — definicja bez odwołania jest tu normą
PY_PLIK_KONWENCJI = re.compile(r'(^|/)(migrations|alembic|commands|management)/|'
                               r'(^|/)(manage|wsgi|asgi|conftest|setup|__main__|settings|urls|admin|apps|signals)\.py$')
# Dekoratory rejestrujące funkcję gdzie indziej (CLI, zadania, sygnały, fixture'y) — bez wywołania po nazwie
PY_DEKORATOR_KONWENCJI = re.compile(r'^@(?:\w+\.)*(shared_task|task|celery|click|command|group|option|argument|'
                                    r'receiver|fixture|on_event|exception_handler|middleware|websocket|'
                                    r'filter|simple_tag|inclusion_tag|tag|'
                                    r'event|listens_for|register|hookimpl|validator|field_validator|model_validator|'
                                    r'property|staticmethod|classmethod|overload|lifespan|subscriber|consumer|'
                                    r'job|scheduled|cron|periodic_task|dataclass_transform|pytest\.\w+|'
                                    r'app\.(?:cli|command|task|errorhandler|before_request|after_request|teardown\w*|'
                                    r'context_processor|template_filter))\b')
PY_TRASA = re.compile(r'^@(\w+)\.(get|post|put|patch|delete|route|api_route|head|options)\(\s*[\'"]([^\'"]*)[\'"]')
PY_PREFIKS = re.compile(r'(?:APIRouter|Blueprint|Router)\([^)]*?(?:prefix|url_prefix)\s*=\s*[\'"]([^\'"]*)[\'"]')
# Rejestracja po nazwie w pliku definicji: `admin.site.register(Model)`, `app.add_command(cmd)` — wejście jest
PY_REJESTRACJA = re.compile(r'\b(register|setup|add_command|add_typer|include_router|add_api_route|add_url_rule|'
                            r'register_blueprint|connect|add_handler|add_middleware|mount)\s*\(')

# --- JS / TS ---------------------------------------------------------------------------
JS_DEF = re.compile(r'^export\s+(?:default\s+)?(?:async\s+)?function\s*\*?\s*(\w+)|'
                    r'^export\s+(?:default\s+)?(?:abstract\s+)?class\s+(\w+)|'
                    r'^export\s+(?:const|let)\s+(\w+)')
JS_EKSPORT_LISTA = re.compile(r'^export\s*\{([^}]*)\}\s*;?\s*$')      # bez `from` — re-eksport to nie definicja
JS_PLIK_POMIJANY = re.compile(r'(^|/)(index|main)\.(ts|tsx|js|jsx|mjs|cjs)$|\.d\.ts$|\.config\.[^/]+$|'
                              r'(^|/)(vite|next|nuxt|svelte|remix|astro|tailwind|postcss|jest|vitest|playwright|'
                              r'eslint|prettier|babel|webpack|rollup|tsup)\.[^/]*(config|rc)[^/]*$|'
                              r'(^|/)(setupTests|vite-env|global)\.[^/]+$')
JS_ROUTING_PLIKOWY = re.compile(r'(^|/)(pages|app|routes)/')
# Które frameworki routują przez pliki — tylko wtedy pages/ app/ routes/ są punktami wejścia
ROUTING_PLIKOWY_CFG = re.compile(r'(^|/)(next|svelte|remix|nuxt|astro|react-router)\.config\.[^/]+$')
ROUTING_PLIKOWY_DEP = re.compile(r'"(next|@sveltejs/kit|@remix-run/[\w-]+|nuxt|astro|@tanstack/react-router|'
                                 r'@tanstack/start|react-router|@react-router/dev|solid-start|qwik-city)"')
JS_TRASA = re.compile(r'\b(?:app|router|server|fastify|hono|api|r)\.(get|post|put|patch|delete|all)\(\s*[\'"`](/[^\'"`]*)[\'"`]')
NEST_TRASA = re.compile(r'^\s*@(Get|Post|Put|Patch|Delete|All|Head|Options)\(\s*(?:[\'"`]([^\'"`]*)[\'"`])?\s*\)')
NEST_KONTROLER = re.compile(r'@Controller\(\s*(?:[\'"`]([^\'"`]*)[\'"`])?')
KOMPONENT_PLIK = re.compile(r'\.(tsx|jsx)$')
KOMPONENT_CONST = re.compile(r'^export\s+(?:const|let)\s+([A-Z]\w*)\s*(?::\s*[\w.<>,\s\[\]|]+)?=\s*'
                             r'(?:\(|async\s*\(|React\.memo|memo|forwardRef|React\.forwardRef|styled|observer|function)')

# --- Go ---------------------------------------------------------------------------------
GO_DEF = re.compile(r'^func\s+([A-Z]\w*)\s*\(')
GO_METODA = re.compile(r'^func\s*\(')
GO_POMIJANE = re.compile(r'^(main|init|Test\w*|Benchmark\w*|Example\w*|Fuzz\w*)$')

# --- Narzędzia -------------------------------------------------------------------------
VULTURE = re.compile(r'^(.+?):(\d+):\s+unused\s+(?:function|class|method|variable|import|property|attribute)\s+'
                     r"'(\w+)'")
TS_PRUNE = re.compile(r'^(.+?):(\d+)\s+-\s+(\w+)')


def odcisk(t):
    return hashlib.sha256(t.encode('utf-8', 'replace')).hexdigest()[:16]


def wypisz(**z):
    print(json.dumps(z, ensure_ascii=False))


def git(root, *args, limit=30):
    try:
        r = subprocess.run(['git', '-C', root, *args], capture_output=True, text=True, timeout=limit)
    except (OSError, subprocess.TimeoutExpired):
        return None
    return r.stdout if r.returncode == 0 else None


def jezyk(p):
    if p.endswith(('.py', '.pyi')):
        return 'py'
    if p.endswith(('.ts', '.tsx', '.js', '.jsx', '.mjs', '.cjs')):
        return 'js'
    if p.endswith('.go'):
        return 'go'
    return None


# --- Punkt odniesienia i diff -----------------------------------------------------------

def punkt_odniesienia(root, punkt):
    """Rewizja, od której symbol jest „nowy". RALPH_OD (CI: origin/main) bije wszystko;
    potem tag fazy / wydania; bez tagu — pierwszy commit (jedyny commit → puste drzewo)."""
    od = os.environ.get('RALPH_OD', '').strip()
    if od and git(root, 'rev-parse', '--verify', '-q', od + '^{commit}'):
        return od
    wzorce = {'wydanie': ['v[0-9]*'], 'faza': ['ralph/faza-*'], 'ci': ['ralph/faza-*'], 'commit': ['ralph/faza-*']}
    for wz in wzorce.get(punkt, ['ralph/faza-*']):
        t = git(root, 'describe', '--tags', '--abbrev=0', '--match', wz, 'HEAD')
        if t and t.strip():
            return t.strip()
    pierwszy = (git(root, 'rev-list', '--max-parents=0', 'HEAD') or '').split()
    head = (git(root, 'rev-parse', 'HEAD') or '').strip()
    if not pierwszy or not head:
        return None
    if head in pierwszy:
        return PUSTE_DRZEWO
    return pierwszy[-1]


def dodane_linie(root, od):
    """{plik: [(nr_linii, treść)]} — linie dodane między punktem odniesienia a HEAD."""
    zakres = f'{od}...HEAD' if od != PUSTE_DRZEWO else f'{od} HEAD'
    args = ['diff', '-U0', '--no-color', '--diff-filter=AMR', '--find-renames', *zakres.split()]
    d = git(root, *args, limit=120)
    if d is None and od != PUSTE_DRZEWO:              # brak merge-base (płytki klon) — zwykły zakres
        d = git(root, 'diff', '-U0', '--no-color', '--diff-filter=AMR', od, 'HEAD', limit=120)
    if not d:
        return {}
    out, plik, nr = {}, None, 0
    for linia in d.split('\n'):
        if linia.startswith('+++ '):
            plik = linia[6:] if linia.startswith('+++ b/') else None
            continue
        if linia.startswith('@@'):
            m = re.match(r'@@ -\d+(?:,\d+)? \+(\d+)', linia)
            nr = int(m.group(1)) if m else 0
            continue
        if plik is None or linia.startswith(('---', 'diff ', 'index ', 'Binary', 'new file', 'deleted', 'similarity',
                                             'rename ', 'old mode', 'new mode')):
            continue
        if linia.startswith('+'):
            out.setdefault(plik, []).append((nr, linia[1:]))
            nr += 1
    return out


# --- Korpus --------------------------------------------------------------------------------

def wczytaj(root, p):
    try:
        if os.path.getsize(os.path.join(root, p)) > MAX_BAJTOW:
            return None
        with open(os.path.join(root, p), 'rb') as f:
            dane = f.read()
    except OSError:
        return None
    if b'\0' in dane[:8192]:
        return None
    return dane.decode('utf-8', 'replace')


def korpus_plikow(pliki):
    return [p for p in pliki if KOD.search(p) and not POMIJANE_KATALOGI.search(p) and not POMIJANE_PLIKI.search(p)]


def routing_plikowy(root, korpus, tresci):
    """Czy projekt routuje przez pliki (Next, SvelteKit, Remix…) — wtedy pages/app/routes to wejścia."""
    if any(ROUTING_PLIKOWY_CFG.search(p) for p in korpus):
        return True
    for p in korpus:
        if os.path.basename(p) == 'package.json' and ROUTING_PLIKOWY_DEP.search(tresci.get(p) or ''):
            return True
    return False


# --- Definicje -------------------------------------------------------------------------------

def normalizuj_trase(sciezka):
    """`/api/v1/items/{id}/` → regex łapiący `/items/5`, `/api/items/${id}`, `/items/:id`.
    Pusta ścieżka albo sam korzeń → None (nie da się ocenić)."""
    s = re.sub(r'^/api(?:/v\d+)?', '', sciezka.strip()).rstrip('/')
    if not s or s == '/':
        return None
    czesci = []
    for seg in s.strip('/').split('/'):
        if re.match(r'^(:\w+|\{[^}]*\}|<[^>]*>|\[[^\]]*\]|\*.*|\$\{.*\})$', seg):
            czesci.append(r'[^/\s\'"`?#]+')
        else:
            czesci.append(re.escape(seg))
    return re.compile(r'(?<![\w/])(?:/api(?:/v\d+)?)?/' + '/'.join(czesci) + r'(?![\w-])')


def definicje_py(p, dodane, wiersze):
    """[(nazwa, nr, rodzaj)] — rodzaj: 'symbol' | ('trasa', ścieżka)."""
    if PY_PLIK_KONWENCJI.search(p):
        return []
    tekst = '\n'.join(wiersze)
    baza = os.path.basename(p)
    # Zadania Celery i komendy CLI rejestrują się dekoratorem — nikt ich nie woła po nazwie
    if baza in ('tasks.py', 'celery.py', 'worker.py') and re.search(r'\bshared_task\b|\bcelery\b', tekst, re.I):
        return []
    if baza in ('cli.py', 'commands.py', '__main__.py') and re.search(r'@click|@app\.command|@cli\.|typer\.', tekst):
        return []
    m_all = re.search(r'^__all__\s*[:=]\s*\[([^\]]*)\]', tekst, re.M | re.S)
    w_all = set(re.findall(r'[\'"](\w+)[\'"]', m_all.group(1))) if m_all else set()
    prefiks = PY_PREFIKS.search(tekst)
    prefiks = prefiks.group(1) if prefiks else ''
    out = []
    for nr, linia in dodane:
        m = PY_DEF.match(linia)
        if not m:
            continue
        nazwa = m.group(1) or m.group(2)
        # Test* to klasa unittest odkrywana po nazwie — jak test_* (TestHTTP w benchmarku Ralpha)
        if nazwa.startswith('_') or nazwa == 'main' or nazwa.startswith('test_') or nazwa in w_all \
                or re.match(r'^Test[A-Z_]', nazwa):
            continue
        # Kontekst (dekoratory) z pliku na dysku — diff -U0 go nie ma. Gdy linia pod numerem
        # nie jest tą definicją (drzewo robocze różni się od HEAD), szukamy jej po treści.
        idx = nr - 1
        if not (0 <= idx < len(wiersze) and wiersze[idx] == linia):
            idx = next((i for i, w in enumerate(wiersze) if w == linia), None)
            if idx is None:
                continue
        dekoratory = []
        i = idx - 1
        while i >= 0 and (wiersze[i].startswith('@') or (dekoratory and wiersze[i].strip() == '')):
            if wiersze[i].startswith('@'):
                dekoratory.append(wiersze[i])
            i -= 1
        if any(PY_DEKORATOR_KONWENCJI.match(dk) for dk in dekoratory):
            continue
        trasa = next((PY_TRASA.match(dk) for dk in dekoratory if PY_TRASA.match(dk)), None)
        if trasa:
            out.append((nazwa, nr, ('trasa', prefiks + trasa.group(3))))
            continue
        if any(PY_REJESTRACJA.search(w) and re.search(r'\b%s\b' % re.escape(nazwa), w)
               for w in wiersze if w != linia):
            continue
        out.append((nazwa, nr, 'symbol'))
    return out


def definicje_js(p, dodane, wiersze, pliki_routing):
    if JS_PLIK_POMIJANY.search(p) or (pliki_routing and JS_ROUTING_PLIKOWY.search(p)):
        return []
    tekst = '\n'.join(wiersze)
    komponenty = bool(KOMPONENT_PLIK.search(p))
    kontroler = NEST_KONTROLER.search(tekst)
    prefiks = ('/' + kontroler.group(1).strip('/')) if kontroler and kontroler.group(1) else ''
    out = []
    for nr, linia in dodane:
        m = JS_TRASA.search(linia)
        if m:
            out.append((m.group(2), nr, ('trasa', m.group(2))))
            continue
        m = NEST_TRASA.match(linia)
        if m and kontroler:
            sc = prefiks + ('/' + m.group(2).strip('/') if m.group(2) else '')
            out.append((sc or '/', nr, ('trasa', sc)))
            continue
        m = JS_EKSPORT_LISTA.match(linia)
        if m:
            for el in m.group(1).split(','):
                el = el.strip()
                if not el or el == 'default' or el.startswith('type '):
                    continue
                nazwa = el.split(' as ')[-1].strip()
                if re.match(r'^\w+$', nazwa) and nazwa != 'default':
                    out.append((nazwa, nr, 'symbol'))
            continue
        m = JS_DEF.match(linia)
        if not m:
            continue
        nazwa = m.group(1) or m.group(2) or m.group(3)
        if nazwa.startswith('_'):
            continue
        if komponenty and nazwa[0].isupper() and (m.group(1) or KOMPONENT_CONST.match(linia)):
            out.append((nazwa, nr, 'komponent'))
        else:
            out.append((nazwa, nr, 'symbol'))
    return out


def definicje_go(p, dodane):
    out = []
    for nr, linia in dodane:
        if GO_METODA.match(linia):
            continue
        m = GO_DEF.match(linia)
        if m and not GO_POMIJANE.match(m.group(1)):
            out.append((m.group(1), nr, 'symbol'))
    return out


# --- Narzędzia -----------------------------------------------------------------------------

def uruchom(cmd, cwd):
    try:
        r = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, timeout=LIMIT_NARZEDZIA)
    except (OSError, subprocess.TimeoutExpired):
        return None
    return r.stdout


def z_vulture(root, pliki_py):
    """{(plik, nazwa)} zgłoszone przez vulture (min. 80% pewności)."""
    if not pliki_py or not shutil.which('vulture'):
        return set()
    out = uruchom(['vulture', *pliki_py, '--min-confidence', '80'], root)
    wynik = set()
    for linia in (out or '').splitlines():
        m = VULTURE.match(linia)
        if m:
            wynik.add((os.path.normpath(m.group(1)), m.group(3)))
    return wynik


def z_knip(root, korpus, tresci):
    """{(plik, nazwa)} z knipa / ts-prune — tylko gdy projekt ma je w devDependencies (nic nie instalujemy)."""
    wynik = set()
    for p in korpus:
        if os.path.basename(p) != 'package.json' or POMIJANE_KATALOGI.search(p):
            continue
        try:
            dev = json.loads(tresci.get(p) or '{}').get('devDependencies') or {}
        except ValueError:
            continue
        katalog = os.path.dirname(p)
        cwd = os.path.join(root, katalog) if katalog else root
        if 'knip' in dev and shutil.which('npx'):
            out = uruchom(['npx', '--no-install', 'knip', '--reporter', 'json', '--no-progress'], cwd)
            try:
                dane = json.loads(out or '')
            except ValueError:
                dane = None
            if isinstance(dane, dict):
                for iss in dane.get('issues') or []:
                    plik = os.path.normpath(os.path.join(katalog, iss.get('file', '')))
                    for klucz in ('exports', 'types', 'nsExports', 'nsTypes', 'classMembers'):
                        for el in iss.get(klucz) or []:
                            nazwa = el.get('name') if isinstance(el, dict) else el
                            if nazwa:
                                wynik.add((plik, ('knip', str(nazwa))))
                for plik in dane.get('files') or []:
                    wynik.add((os.path.normpath(os.path.join(katalog, plik)), ('knip', '*')))
        if 'ts-prune' in dev and shutil.which('npx'):
            out = uruchom(['npx', '--no-install', 'ts-prune'], cwd)
            for linia in (out or '').splitlines():
                m = TS_PRUNE.match(linia)
                if m:
                    wynik.add((os.path.normpath(os.path.join(katalog, m.group(1))), ('ts-prune', m.group(3))))
    return wynik


# --- Główny przebieg -----------------------------------------------------------------------

def sprawdz(root, pliki):
    """--sprawdz: tryb wbudowany jest normalny (bez uwagi). Uwaga tylko wtedy, gdy projekt
    deklaruje narzędzie, a nie ma go ani w PATH, ani w node_modules/.bin obok package.json."""
    # Z surowej listy, nie z korpusu: requirements*.txt nie jest plikiem kodu, a deklaruje vulture
    tresci = {p: wczytaj(root, p) or '' for p in pliki
              if not POMIJANE_KATALOGI.search(p) and (
                  os.path.basename(p) in ('package.json', 'pyproject.toml', 'setup.cfg', 'tox.ini',
                                          '.pre-commit-config.yaml') or os.path.basename(p).startswith('requirements'))}
    brak = []
    for p, t in tresci.items():
        if os.path.basename(p) == 'package.json':
            try:
                dev = json.loads(t).get('devDependencies') or {}
            except ValueError:
                continue
            for n in ('knip', 'ts-prune'):
                lokalny = os.path.join(root, os.path.dirname(p), 'node_modules', '.bin', n)
                if n in dev and not (shutil.which(n) or os.path.exists(lokalny)):
                    brak.append(n)
        elif re.search(r'\bvulture\b', t) and not shutil.which('vulture'):
            brak.append('vulture')
    if brak:
        print(f'uwaga: projekt ma {", ".join(sorted(set(brak)))} w zależnościach, a w PATH ich nie ma — '
              f'działa tryb wbudowany (grep po definicjach); zainstaluj, żeby dołożyć sygnał narzędzia')
    return 0


def main():
    root = os.environ.get('RALPH_ROOT') or os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
    if '--sprawdz' in sys.argv:
        pliki = (git(root, 'ls-files') or '').splitlines()
        return sprawdz(root, [p for p in pliki if p.strip()])
    punkt = os.environ.get('RALPH_PUNKT', 'faza')
    pliki = [l.strip() for l in sys.stdin.read().splitlines() if l.strip()]
    if not pliki:
        return 0
    od = punkt_odniesienia(root, punkt)
    if not od:
        return 0                                  # bez gita nie ma „nowych" — nic do sprawdzenia
    dodane = dodane_linie(root, od)
    if not dodane:
        return 0

    korpus = korpus_plikow(pliki)
    zbior = set(korpus)
    tresci = {}
    for p in korpus:
        t = wczytaj(root, p)
        if t is not None:
            tresci[p] = t
    ma_front = any(MA_FRONT.search(p) for p in korpus)
    pliki_routing = routing_plikowy(root, korpus, tresci)

    # 1. Nowe definicje w plikach kodu z zakresu (poza testami, stories, wygenerowanymi katalogami)
    definicje = []                                 # (plik, nazwa, nr, rodzaj)
    for p, linie in dodane.items():
        if p not in zbior or p not in tresci or PLIK_TESTU.search(p) or STORIES.search(p):
            continue
        jez = jezyk(p)
        wiersze = tresci[p].split('\n')
        if jez == 'py':
            defs = definicje_py(p, linie, wiersze)
        elif jez == 'js':
            defs = definicje_js(p, linie, wiersze, pliki_routing)
        elif jez == 'go':
            defs = definicje_go(p, linie)
        else:
            continue
        for nazwa, nr, rodzaj in defs:
            if isinstance(rodzaj, tuple) and not ma_front:
                continue                           # sam backend — klient trasy jest gdzie indziej
            definicje.append((p, nazwa, nr, rodzaj))
    if not definicje:
        return 0

    # 2. Odwołania: jeden przebieg po korpusie z alternatywą wszystkich nazw (nie N × korpus)
    nazwy = sorted({d[1] for d in definicje if not isinstance(d[3], tuple)}, key=len, reverse=True)
    wz_nazw = re.compile(r'\b(' + '|'.join(map(re.escape, nazwy)) + r')\b') if nazwy else None
    trasy = {}
    for p, nazwa, nr, rodzaj in definicje:
        if isinstance(rodzaj, tuple):
            wz = normalizuj_trase(rodzaj[1])
            if wz:
                trasy[(p, nazwa)] = wz
    odwolania = {}                                 # nazwa → {pliki z wystąpieniem}
    odwolania_tras = {k: set() for k in trasy}
    # Wystąpienie w pliku definicji liczy się, gdy pada w innej linii niż definicja (i nie w komentarzu):
    # pierwszy pomiar na repo Ralpha dał 146 „martwych" funkcji, z czego każda była helperem wołanym
    # we własnym module. Toolbar z retro nadal wpada — jego nazwa stoi tylko w linii `def`.
    linie_def = {}                                 # (plik, nazwa) → {numery linii definicji}
    for p, nazwa, nr, rodzaj in definicje:
        if not isinstance(rodzaj, tuple):
            linie_def.setdefault((p, nazwa), set()).add(nr)
    for p, t in tresci.items():
        if PLIK_TESTU.search(p) or STORIES.search(p):
            continue
        if wz_nazw:
            w_tym_pliku = {k[1] for k in linie_def if k[0] == p}
            if not w_tym_pliku:
                for m in wz_nazw.finditer(t):
                    odwolania.setdefault(m.group(1), set()).add(p)
            else:
                for nr, linia in enumerate(t.split('\n'), 1):
                    if linia.lstrip().startswith(('#', '//', '*', '/*')):
                        continue
                    for m in wz_nazw.finditer(linia):
                        n = m.group(1)
                        if n in w_tym_pliku and (nr in linie_def[(p, n)] or re.match(
                                r'^\s*(?:export\s+)?(?:default\s+)?(?:async\s+)?(?:def|class|function\s*\*?|const|let|func)\s+'
                                + re.escape(n) + r'\b', linia)):
                            continue
                        odwolania.setdefault(n, set()).add(p)
        if trasy and FRONT.search(p):
            for k, wz in trasy.items():
                if k[0] != p and wz.search(t):
                    odwolania_tras[k].add(p)

    # 3. Narzędzia jako dodatkowy sygnał (wyłącznie dla symboli nowych w diffie)
    pliki_py = sorted({d[0] for d in definicje if jezyk(d[0]) == 'py'})
    narzedzia = {(pl, ('vulture', n)) for pl, n in z_vulture(root, pliki_py)} | z_knip(root, korpus, tresci)
    wg_symbolu = {}
    for pl, (narz, n) in narzedzia:
        wg_symbolu.setdefault((pl, n), set()).add(narz)
        if n == '*':
            wg_symbolu.setdefault((pl, '*'), set()).add(narz)

    # 4. Znaleziska
    znaleziska = []
    for p, nazwa, nr, rodzaj in sorted(definicje, key=lambda d: (d[0], d[2])):
        narz = sorted(wg_symbolu.get((p, nazwa), set()) | wg_symbolu.get((p, '*'), set()))
        if isinstance(rodzaj, tuple):
            k = (p, nazwa)
            if k not in trasy or odwolania_tras[k]:
                continue
            znaleziska.append(dict(plik=p, linia=nr, regula='trasa-bez-klienta',
                                   opis=f'trasa `{rodzaj[1]}` dodana w tej fazie — żaden plik frontu / klienta / '
                                        f'dokumentacji API jej nie woła; dopnij wejście (AC: wejście) albo usuń',
                                   odcisk_tresci=odcisk(f'{p}:{rodzaj[1]}')))
            continue
        if odwolania.get(nazwa):                   # w pliku definicji liczone już tylko linie poza definicją
            continue
        regula = 'komponent-bez-rodzica' if rodzaj == 'komponent' else 'symbol-bez-wejscia'
        co = 'komponent' if rodzaj == 'komponent' else 'symbol'
        potw = f' (potwierdza: {", ".join(narz)})' if narz else ''
        czego = 'nie renderuje' if rodzaj == 'komponent' else 'nie wywołuje'
        znaleziska.append(dict(plik=p, linia=nr, regula=regula,
                               opis=f'{co} `{nazwa}` dodany w tej fazie — poza własnym testem nic go {czego}{potw}; '
                                    f'dopnij wejście (AC: wejście) albo usuń',
                               odcisk_tresci=odcisk(f'{p}:{nazwa}')))
    # Narzędzie widzi martwy symbol, którego nasz grep nie złapał (np. nazwa trafiona w stringu) — osobna reguła
    zgloszone = {(z['plik'], z['odcisk_tresci']) for z in znaleziska}
    for p, nazwa, nr, rodzaj in sorted(definicje, key=lambda d: (d[0], d[2])):
        if isinstance(rodzaj, tuple) or (p, odcisk(f'{p}:{nazwa}')) in zgloszone:
            continue
        narz = sorted(wg_symbolu.get((p, nazwa), set()))
        if narz:
            znaleziska.append(dict(plik=p, linia=nr, regula='martwy-symbol-narzedzie',
                                   opis=f'`{nazwa}` dodany w tej fazie — {", ".join(narz)} nie widzi żadnego użycia '
                                        f'(nasz grep znalazł nazwę gdzie indziej, może w stringu); sprawdź wejście',
                                   odcisk_tresci=odcisk(f'{p}:{nazwa}')))

    for z in znaleziska[:MAX_ZNALEZISK]:
        wypisz(waga='ostrzega', **z)
    reszta = znaleziska[MAX_ZNALEZISK:]
    if reszta:
        lista = ', '.join(f'{z["plik"]}:{z["linia"]}' for z in reszta[:8]) + (' …' if len(reszta) > 8 else '')
        wypisz(plik=reszta[0]['plik'], linia=reszta[0]['linia'], regula='symbol-bez-wejscia', waga='ostrzega',
               odcisk_pliku=False,
               opis=f'+{len(reszta)} dalszych symboli bez wejścia ({lista}) — pełna lista: '
                    f'python3 ralph-kontrole/ralph-kontrole.py --punkt faza',
               odcisk_tresci='licznik')
    return 0


if __name__ == '__main__':
    sys.exit(main())
