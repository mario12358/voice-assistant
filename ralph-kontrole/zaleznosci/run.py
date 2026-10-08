#!/usr/bin/env python3
"""Kontrola `zaleznosci` — nowe pakiety w manifestach vs rejestry (npm, PyPI, JSR).

Lista plików na stdin; z niej bierze tylko manifesty. Pakiet jest „nowy", gdy nie było
go w wersji pliku z HEAD. Znaleziska JSON w liniach na stdout.
"""
import datetime
import json
import os
import re
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), '..'))
from _siec import BrakSieci, Pamiec, adres, zapytaj  # noqa: E402

try:
    import tomllib
except ImportError:          # Python < 3.11 — pyproject.toml pomijamy, reszta działa
    tomllib = None

WIEK_BLOKUJE = 7
WIEK_OSTRZEGA = 30
POBRANIA_MALO = 50
UDZIAL_LITEROWKI = 0.01

# Najczęściej używane pakiety — wzorzec dla literówek. Krótka lista celowo: literówka
# w nazwie, której nikt nie używa, nikogo nie skusi.
POPULARNE = {
    'npm': '''react react-dom next vue svelte angular express fastify koa hono axios lodash underscore
        moment dayjs date-fns uuid nanoid zod yup joi ajv dotenv chalk commander yargs debug
        typescript ts-node tsx esbuild vite webpack rollup babel-core eslint prettier jest vitest
        mocha chai sinon supertest playwright puppeteer cypress jsonwebtoken bcrypt bcryptjs
        passport cors helmet body-parser cookie-parser multer mongoose sequelize prisma typeorm
        knex pg mysql mysql2 sqlite3 redis ioredis socket.io ws graphql apollo-server rxjs
        tailwindcss postcss autoprefixer sass styled-components classnames clsx immer zustand
        redux react-redux react-router react-router-dom swr node-fetch cross-fetch form-data
        semver glob rimraf mkdirp fs-extra minimist inquirer ora execa nodemon concurrently
        cross-env husky lint-staged pino winston bunyan morgan sharp jimp cheerio jsdom
        marked markdown-it highlight.js crypto-js nodemailer stripe openai i18next'''.split(),
    'pypi': '''requests urllib3 httpx aiohttp flask django fastapi starlette uvicorn gunicorn
        pydantic sqlalchemy alembic psycopg2 psycopg2-binary psycopg asyncpg pymysql redis celery
        numpy pandas scipy matplotlib seaborn scikit-learn torch tensorflow keras transformers
        pillow opencv-python beautifulsoup4 lxml selenium playwright pytest pytest-cov
        pytest-asyncio mock coverage tox black ruff flake8 pylint mypy isort pre-commit
        python-dotenv pyyaml toml click typer rich tqdm jinja2 markupsafe werkzeug itsdangerous
        cryptography pyjwt bcrypt passlib paramiko boto3 botocore awscli google-cloud-storage
        azure-storage-blob openai anthropic tiktoken langchain python-dateutil pytz arrow
        attrs dataclasses-json marshmallow orjson ujson simplejson six setuptools wheel pip
        certifi idna charset-normalizer chardet docker kubernetes grpcio protobuf sentry-sdk
        structlog loguru prometheus-client opentelemetry-api tomli typing-extensions anyio'''.split(),
}


def wypisz(**z):
    print(json.dumps(z, ensure_ascii=False))


# --- Parsowanie manifestów -------------------------------------------------------

def norm_py(n):
    return re.sub(r'[-_.]+', '-', n).lower()


def spoza_rejestru(spec):
    s = spec.strip()
    return bool(re.match(r'^(git\+|git:|github:|gitlab:|bitbucket:|https?:|ssh:)', s)
                or re.match(r'^[\w.-]+/[\w.-]+(#.*)?$', s))     # npm: user/repo


def z_package_json(tekst):
    """{nazwa: spec} ze wszystkich grup zależności."""
    try:
        d = json.loads(tekst)
    except ValueError:
        return {}
    out = {}
    for grupa in ('dependencies', 'devDependencies', 'optionalDependencies', 'peerDependencies'):
        for n, spec in (d.get(grupa) or {}).items():
            if isinstance(spec, str):
                out[n] = spec
    return out


def z_requirements(tekst):
    """({nazwa: spec}, dodatkowe indeksy)."""
    out, indeksy = {}, []
    for linia in tekst.splitlines():
        l = linia.split(' #', 1)[0].strip()
        if not l or l.startswith('#'):
            continue
        if re.match(r'^(--extra-index-url|--index-url|-i)\b', l):
            indeksy.append(l)
            continue
        if l.startswith('-e ') or l.startswith('--editable'):
            cel = l.split(None, 1)[1] if ' ' in l else ''
            m = re.search(r'#egg=([\w.-]+)', cel)
            if m and spoza_rejestru(cel):
                out[norm_py(m.group(1))] = cel
            continue
        if l.startswith('-'):
            continue
        m = re.match(r'^([A-Za-z0-9][A-Za-z0-9._-]*)(\[[^\]]*\])?\s*(.*)$', l)
        if m:
            reszta = m.group(3).strip()
            out[norm_py(m.group(1))] = reszta[1:].strip() if reszta.startswith('@') else reszta
    return out, indeksy


def nazwa_pep508(s):
    m = re.match(r'^\s*([A-Za-z0-9][A-Za-z0-9._-]*)(\[[^\]]*\])?\s*(@\s*(\S+))?', s)
    if not m:
        return None, ''
    return norm_py(m.group(1)), (m.group(4) or '')


def z_pyproject(tekst):
    if tomllib is None:
        return {}
    try:
        d = tomllib.loads(tekst)
    except Exception:
        return {}
    out = {}
    proj = d.get('project') or {}
    listy = [proj.get('dependencies') or []]
    listy += list((proj.get('optional-dependencies') or {}).values())
    listy += [g for g in (d.get('dependency-groups') or {}).values()]
    for lista in listy:
        for s in lista:
            if isinstance(s, str):
                n, url = nazwa_pep508(s)
                if n:
                    out[n] = url
    poetry = (d.get('tool') or {}).get('poetry') or {}
    grupy = [poetry.get('dependencies') or {}, poetry.get('dev-dependencies') or {}]
    grupy += [g.get('dependencies') or {} for g in (poetry.get('group') or {}).values()]
    for g in grupy:
        for n, spec in g.items():
            if n.lower() == 'python':
                continue
            if isinstance(spec, dict) and (spec.get('git') or spec.get('url')):
                out[norm_py(n)] = spec.get('git') or spec.get('url')
            elif isinstance(spec, dict) and spec.get('path'):
                continue
            else:
                out[norm_py(n)] = spec if isinstance(spec, str) else ''
    for n, src in (((d.get('tool') or {}).get('uv') or {}).get('sources') or {}).items():
        if isinstance(src, dict) and (src.get('git') or src.get('url')):
            out[norm_py(n)] = src.get('git') or src.get('url')
    return out


def bez_komentarzy_jsonc(t):
    t = re.sub(r'/\*.*?\*/', '', t, flags=re.S)
    return re.sub(r'(?m)^\s*//.*$|(?<=[,{\[\s])//[^\n"]*$', '', t)


def z_deno(tekst):
    """{(ekosystem, nazwa): spec} z mapy imports."""
    try:
        d = json.loads(bez_komentarzy_jsonc(tekst))
    except ValueError:
        return {}
    out = {}
    for spec in (d.get('imports') or {}).values():
        if not isinstance(spec, str):
            continue
        m = re.match(r'^(npm|jsr):(@?[^@/]+(?:/[^@/]+)?)(?:@.*)?$', spec)
        if m:
            out[('npm' if m.group(1) == 'npm' else 'jsr', m.group(2))] = spec
        elif spec.startswith('http'):
            out[('url', spec)] = spec
    return out


def manifest(sciezka, tekst):
    """[(ekosystem, nazwa, spec)], [ostrzeżenia o indeksach]"""
    base = os.path.basename(sciezka)
    if base == 'package.json':
        return [('npm', n, s) for n, s in z_package_json(tekst).items()], []
    if re.match(r'^requirements.*\.(txt|in)$', base):
        d, indeksy = z_requirements(tekst)
        return [('pypi', n, s) for n, s in d.items()], indeksy
    if base == 'pyproject.toml':
        return [('pypi', n, s) for n, s in z_pyproject(tekst).items()], []
    if base in ('deno.json', 'deno.jsonc'):
        return [(e, n, s) for (e, n), s in z_deno(tekst).items()], []
    return None, []


def z_head(root, sciezka):
    try:
        r = subprocess.run(['git', '-C', root, 'show', f'HEAD:{sciezka}'], capture_output=True,
                           text=True, timeout=10)
    except (OSError, subprocess.TimeoutExpired):
        return ''
    return r.stdout if r.returncode == 0 else ''


def prywatne_scope(root):
    """Scope'y npm z własnym rejestrem w .npmrc — ich pakietów nie ma w publicznym."""
    try:
        with open(os.path.join(root, '.npmrc'), encoding='utf-8') as f:
            return set(re.findall(r'^\s*(@[\w.-]+):registry\s*=', f.read(), re.M))
    except OSError:
        return set()


# --- Rejestry ----------------------------------------------------------------------

def dni_od(iso):
    try:
        t = datetime.datetime.fromisoformat(iso.replace('Z', '+00:00'))
    except (ValueError, AttributeError):
        return None
    if t.tzinfo is None:
        t = t.replace(tzinfo=datetime.timezone.utc)
    return (datetime.datetime.now(datetime.timezone.utc) - t).days


def npm_info(nazwa):
    kod, d = zapytaj(f'{adres("npm")}/{nazwa.replace("/", "%2F")}')
    if kod == 404 or not d:
        return {'istnieje': False}
    latest = (d.get('dist-tags') or {}).get('latest')
    wersja = (d.get('versions') or {}).get(latest) or {}
    skrypty = sorted(k for k in (wersja.get('scripts') or {}) if k in ('preinstall', 'install', 'postinstall'))
    info = {'istnieje': True, 'utworzony': (d.get('time') or {}).get('created'),
            'skrypty': skrypty, 'przestarzaly': bool(wersja.get('deprecated'))}
    try:
        kod, p = zapytaj(f'{adres("npm_pobrania")}/{nazwa}')
        info['pobrania'] = (p or {}).get('downloads') if kod != 404 else 0
    except BrakSieci:
        info['pobrania'] = None
    return info


def pypi_info(nazwa):
    kod, d = zapytaj(f'{adres("pypi")}/{nazwa}/json')
    if kod == 404 or not d:
        return {'istnieje': False}
    czasy = [f.get('upload_time_iso_8601') for pliki in (d.get('releases') or {}).values()
             for f in pliki if f.get('upload_time_iso_8601')]
    return {'istnieje': True, 'utworzony': min(czasy) if czasy else None}


def jsr_info(nazwa):
    m = re.match(r'^@([^/]+)/(.+)$', nazwa)
    if not m:
        return {'istnieje': False}
    kod, d = zapytaj(f'{adres("jsr")}/scopes/{m.group(1)}/packages/{m.group(2)}')
    if kod == 404 or not d:
        return {'istnieje': False}
    return {'istnieje': True, 'utworzony': d.get('createdAt')}


INFO = {'npm': npm_info, 'pypi': pypi_info, 'jsr': jsr_info}
REJESTR = {'npm': 'npm', 'pypi': 'PyPI', 'jsr': 'JSR'}


def levenshtein(a, b):
    if abs(len(a) - len(b)) > 2:
        return 3
    poprz = list(range(len(b) + 1))
    for i, ca in enumerate(a, 1):
        biez = [i]
        for j, cb in enumerate(b, 1):
            biez.append(min(poprz[j] + 1, biez[j - 1] + 1, poprz[j - 1] + (ca != cb)))
        poprz = biez
    return poprz[-1]


def podobny(eko, nazwa):
    if nazwa.startswith('@'):
        return None      # scope to osobna przestrzeń nazw; `@nestjs/core` to nie literówka `cors`
    lista = POPULARNE.get('pypi' if eko == 'pypi' else 'npm', [])
    goly = nazwa.lower()
    if goly in lista or nazwa.lower() in lista:
        return None
    for p in lista:
        prog = 1 if len(p) < 8 else 2
        if len(goly) >= 4 and levenshtein(goly, p) <= prog:
            return p
        if re.sub(r'[-_.]', '', goly) == re.sub(r'[-_.]', '', p):
            return p
    return None


# --- Główne ---------------------------------------------------------------------------

def main():
    root = os.environ.get('RALPH_ROOT') or os.getcwd()
    pliki = [l.strip() for l in sys.stdin.read().splitlines() if l.strip()]
    nowe = {}            # (eko, nazwa) → (plik, spec)
    for p in pliki:
        base = os.path.basename(p)
        if base == '.npmrc':
            try:
                with open(os.path.join(root, p), encoding='utf-8') as f:
                    tekst = f.read()
            except OSError:
                continue
            for m in re.finditer(r'^\s*registry\s*=\s*(\S+)', tekst, re.M):
                if 'registry.npmjs.org' not in m.group(1):
                    wypisz(plik=p, linia=None, regula='inny-rejestr', waga='ostrzega',
                           opis=f'domyślny rejestr npm zmieniony na {m.group(1)} — sprawdź, że to zamierzone',
                           odcisk_tresci=m.group(1))
            continue
        try:
            with open(os.path.join(root, p), encoding='utf-8') as f:
                tekst = f.read()
        except OSError:
            continue
        teraz, indeksy = manifest(p, tekst)
        if teraz is None:
            continue
        stare, _ = manifest(p, z_head(root, p))
        bylo = {(e, n) for e, n, _ in (stare or [])}
        for ix in indeksy:
            wypisz(plik=p, linia=None, regula='dodatkowy-indeks', waga='ostrzega',
                   opis=f'„{ix.split()[0]}" — pakiet o tej samej nazwie w dwóch indeksach to droga '
                        f'dependency confusion; upewnij się, że nazwy wewnętrzne są zarezerwowane',
                   odcisk_tresci=ix)
        for eko, nazwa, spec in teraz:
            if (eko, nazwa) not in bylo:
                nowe[(eko, nazwa)] = (p, spec)

    prywatne = prywatne_scope(root)
    do_sprawdzenia = []
    for (eko, nazwa), (plik, spec) in sorted(nowe.items()):
        if eko == 'url' or spoza_rejestru(spec or ''):
            wypisz(plik=plik, linia=None, regula='spoza-rejestru', waga='ostrzega',
                   opis=f'{nazwa}: zależność spoza rejestru ({(spec or nazwa)[:80]}) — bez historii '
                        f'wersji i bez kontroli rejestru; przypnij do commita/sumy kontrolnej',
                   odcisk_tresci=f'{eko}:{nazwa}')
            continue
        if eko == 'npm' and nazwa.startswith('@') and nazwa.split('/')[0] in prywatne:
            continue
        if re.match(r'^(file|link|workspace|portal|patch):', spec or ''):
            continue
        do_sprawdzenia.append((eko, nazwa, plik))
    if not do_sprawdzenia:
        return 0

    pamiec = Pamiec('rejestry')

    def sprawdz(el):
        eko, nazwa, _ = el
        klucz = f'{eko}:{nazwa}'
        z_pamieci = pamiec.daj(klucz)
        if z_pamieci is not None:
            return el, z_pamieci
        try:
            info = INFO[eko](nazwa)
        except BrakSieci:
            return el, None
        pamiec.wstaw(klucz, info, 86400 if info.get('istnieje') else 3600)
        return el, info

    niesprawdzone = []
    with ThreadPoolExecutor(max_workers=8) as ex:
        wyniki = list(ex.map(sprawdz, do_sprawdzenia))

    def pobrania_oryginalu(para):
        eko, wzor = para
        klucz = f'pobrania:{eko}:{wzor}'
        z = pamiec.daj(klucz)
        if z is not None:
            return para, z
        try:
            kod, d = zapytaj(f'{adres("npm_pobrania")}/{wzor}')
        except BrakSieci:
            return para, None
        n = (d or {}).get('downloads') if kod != 404 else None
        if n is not None:
            pamiec.wstaw(klucz, n, 7 * 86400)
        return para, n

    wzory = {(eko, w) for (eko, nazwa, _), info in wyniki
             if eko == 'npm' and info and info.get('istnieje') for w in [podobny(eko, nazwa)] if w}
    with ThreadPoolExecutor(max_workers=4) as ex:
        pobrania_wzoru = dict(ex.map(pobrania_oryginalu, sorted(wzory)))
    pamiec.zapisz()

    for (eko, nazwa, plik), info in wyniki:
        rej = REJESTR[eko]
        odc = f'{eko}:{nazwa}'
        if info is None:
            niesprawdzone.append(nazwa)
            continue
        if not info.get('istnieje'):
            wypisz(plik=plik, linia=None, regula='nie-istnieje', waga='blokuje',
                   opis=f'{nazwa}: pakietu nie ma w rejestrze {rej} — nazwa wymyślona albo literówka; '
                        f'taką nazwę może zarejestrować każdy',
                   odcisk_tresci=odc + ':nie-istnieje')
            continue
        wiek = dni_od(info.get('utworzony'))
        pobrania = info.get('pobrania')
        wzor = podobny(eko, nazwa)
        if wzor:
            # Typosquat ma pobrania właśnie dlatego, że ludzie się mylą (`expres`: ~8,5 tys./tydz.
            # przy 169 mln `express`) — bezwzględny próg go przepuszczał. Liczy się stosunek do
            # oryginału: < 1% jego pobrań to pakiet, który żyje z literówek.
            oryginal = pobrania_wzoru.get((eko, wzor))
            maly_udzial = (pobrania is not None and oryginal and pobrania < oryginal * UDZIAL_LITEROWKI)
            podejrzany = ((wiek is not None and wiek < 90) or (pobrania is not None and pobrania < 1000)
                          or bool(maly_udzial))
            wypisz(plik=plik, linia=None, regula='podobny-do-popularnego',
                   waga='blokuje' if podejrzany else 'ostrzega',
                   opis=f'{nazwa}: nazwa łudząco podobna do „{wzor}"'
                        + (f' (pakiet ma {wiek} dni' + (f', {pobrania} pobrań/tydz.' if pobrania is not None else '')
                           + (f' wobec {oryginal} oryginału' if oryginal else '') + ')'
                           if podejrzany else '') + ' — czy na pewno ten pakiet?',
                   odcisk_tresci=odc + ':podobny')
        if wiek is not None and wiek < WIEK_BLOKUJE:
            wypisz(plik=plik, linia=None, regula='swiezy-pakiet', waga='blokuje',
                   opis=f'{nazwa}: pakiet istnieje w {rej} od {wiek} dni — za młody, żeby mu ufać; '
                        f'poczekaj albo poproś człowieka o wyjątek',
                   odcisk_tresci=odc + ':swiezy')
        elif wiek is not None and wiek < WIEK_OSTRZEGA:
            wypisz(plik=plik, linia=None, regula='mlody-pakiet', waga='ostrzega',
                   opis=f'{nazwa}: pakiet ma {wiek} dni', odcisk_tresci=odc + ':mlody')
        if pobrania is not None and pobrania < POBRANIA_MALO:
            wypisz(plik=plik, linia=None, regula='malo-pobran', waga='ostrzega',
                   opis=f'{nazwa}: {pobrania} pobrań w ostatnim tygodniu — mało kto go używa',
                   odcisk_tresci=odc + ':pobrania')
        if info.get('skrypty'):
            wypisz(plik=plik, linia=None, regula='skrypty-instalacyjne', waga='ostrzega',
                   opis=f'{nazwa}: {", ".join(info["skrypty"])} — kod uruchamiany przy instalacji',
                   odcisk_tresci=odc + ':skrypty')
        if info.get('przestarzaly'):
            wypisz(plik=plik, linia=None, regula='przestarzaly', waga='ostrzega',
                   opis=f'{nazwa}: najnowsza wersja oznaczona w {rej} jako przestarzała',
                   odcisk_tresci=odc + ':przestarzaly')

    if niesprawdzone:
        wypisz(plik=do_sprawdzenia[0][2], linia=None, regula='nie-sprawdzono', waga='ostrzega',
               opis=f'rejestr nieosiągalny — nie sprawdzono: {", ".join(niesprawdzone[:10])}'
                    + (' …' if len(niesprawdzone) > 10 else ''),
               odcisk_tresci='nie-sprawdzono:' + ','.join(sorted(niesprawdzone)))
    return 0


if __name__ == '__main__':
    sys.exit(main())
