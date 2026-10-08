#!/usr/bin/env python3
"""Kontrola `lint-typy` — linter i typechecker projektu jako strażnik + cudzysłów typograficzny.

Lista plików na stdin, znaleziska JSON w liniach na stdout. Komendy bierze z `## Testy`
w ralph/config.md (`Linter/formatter`, `Komenda typów`); reguła cudzysłowu działa bez
żadnego narzędzia. Hook nigdy nie zmienia plików: `--fix` jest zdejmowane, formattery
dostają `--check`.
"""
import hashlib
import json
import os
import re
import shlex
import shutil
import subprocess
import sys
import time

PY = ('py',)
JS = ('js', 'jsx', 'ts', 'tsx', 'mjs', 'cjs', 'vue', 'svelte')
# Granica słowa, która rozdziela też `_` i `-`: `fake_ruff.py` i `ruff-wrapper` to nadal ruff
G = r'(?<![A-Za-z0-9])', r'(?![A-Za-z0-9])'
# (wzorzec w komendzie, rozszerzenia, czy dostaje listę plików, flaga JSON, parser)
NARZEDZIA = [
    ('ruff', PY, True, ['--output-format', 'json'], 'ruff'),
    ('flake8', PY, True, None, 'tekst'),
    ('pylint', PY, True, ['--output-format=json'], 'pylint'),
    ('black', PY, True, None, 'format'),
    ('isort', PY, True, None, 'format'),
    ('eslint', JS, True, ['-f', 'json'], 'eslint'),
    ('biome', JS, True, ['--reporter=json'], 'biome'),
    ('prettier', JS, True, None, 'format'),
    ('golangci-lint', ('go',), False, ['--out-format', 'json'], 'golangci'),
    (r'go\s+vet', ('go',), False, None, 'tekst'),
    (r'cargo\s+clippy', ('rs',), False, ['--message-format', 'json'], 'cargo'),
    (r'cargo\s+fmt', ('rs',), False, None, 'format'),
    ('rubocop', ('rb',), True, ['--format', 'json'], 'rubocop'),
    ('shellcheck', ('sh', 'bash'), True, ['-f', 'json'], 'shellcheck'),
]
NARZEDZIA = [(re.compile(G[0] + wz + G[1]), *reszta) for wz, *reszta in NARZEDZIA]
# Formattery: bez `--check` hook przepisałby pliki przed commitem
FORMATTER_CHECK = {'black': '--check', 'isort': '--check-only', 'prettier': '--check', 'fmt': '--check',
                   'format': '--check'}
LIMIT_COMMIT = 15
LIMIT_PELNY = 240
BUDZET = {'commit': 18, 'faza': 270, 'wydanie': 270, 'ci': 270}   # runner daje 20 s / 300 s na moduł
MAX_ZNALEZISK = 100
MAX_LISTA = 500          # powyżej: narzędzie idzie bez listy (własny zakres), nie ryzykujemy ARG_MAX
START = time.monotonic()

# --- Cudzysłów typograficzny ---------------------------------------------------------
TYPO = '„”“‚’‹›«»'
KOD = re.compile(r'\.(py|js|jsx|ts|tsx|mjs|cjs|go|rs|java|kt|rb|php|sh|bash|sql|ya?ml|toml|json)$')
KOMENTARZ_LINII = {'py': ('#',), 'rb': ('#',), 'sh': ('#',), 'bash': ('#',), 'yaml': ('#',), 'yml': ('#',),
                   'toml': ('#',), 'php': ('//', '#'), 'sql': ('--',), 'json': ()}
KOMENTARZ_BLOKU = {'js', 'jsx', 'ts', 'tsx', 'mjs', 'cjs', 'go', 'rs', 'java', 'kt', 'php', 'sql'}
POTROJNY = {'py'}
BACKTICK = {'js', 'jsx', 'ts', 'tsx', 'mjs', 'cjs', 'go'}
# Języki, w których zwykły string nie może przejść do następnej linii — tam `"tekst”`
# (otwarty ASCII, „zamknięty" typograficznie) to zawsze błąd składni
JEDNOLINIOWE = {'py', 'js', 'jsx', 'ts', 'tsx', 'mjs', 'cjs', 'go', 'java', 'kt', 'toml', 'json', 'yaml', 'yml',
                'sh', 'bash'}
# W shellu i YAML cudzysłów typograficzny poza stringiem to legalny znak (heredoc, tłumaczenia
# w i18n) — zostaje tylko wariant mieszany i przypisanie `X=„…”`
TYLKO_MIESZANE = {'sh', 'bash', 'yaml', 'yml'}
# JSX / PHP mają tekst poza stringami (`<p>Powiedział „tak”, </p>`) — bez reguły zamykającej
BEZ_ZAMYKAJACEGO = {'jsx', 'tsx', 'php'}
OTWIERAJACE = ('=', '(', '[', '{', ',', ':', '+')
ZAMYKAJACE = ')]},;'
SLOWO_OTW = re.compile(r'\b(return|yield)$')
OPIS_TYPO = 'cudzysłów typograficzny jako ogranicznik stringu — zamień na " (pułapka, która wróciła 9 razy na projekcie referencyjnym)'


def odcisk(t):
    return hashlib.sha256(t.encode('utf-8', 'replace')).hexdigest()[:16]


def wypisz(**z):
    print(json.dumps(z, ensure_ascii=False))


def rozszerzenie(p):
    m = KOD.search(p)
    return m.group(1) if m else None


def komentarz_od(s, i, ext):
    """Czy w pozycji i zaczyna się komentarz liniowy."""
    for k in KOMENTARZ_LINII.get(ext, ('//',)):
        if s.startswith(k, i):
            # w shellu / YAML `#` jest komentarzem tylko na początku słowa (`$#`, `a#b` nie)
            if k == '#' and ext in ('sh', 'bash', 'yaml', 'yml') and i > 0 and not s[i - 1].isspace():
                return False
            return True
    return False


def typo_w_linii(s, ext, stan):
    """Numer kolumny pierwszego cudzysłowu typograficznego w roli ogranicznika albo None.
    `stan` niesie komentarz blokowy / string wielolinijkowy między liniami."""
    i, n = 0, len(s)
    w_str, typo_w_str = None, False
    while i < n:
        c = s[i]
        if stan['blok']:
            if s.startswith('*/', i):
                stan['blok'] = False
                i += 2
            else:
                i += 1
            continue
        if stan['wielo']:
            if c == '\\':
                i += 2
            elif s.startswith(stan['wielo'], i):
                i += len(stan['wielo'])
                stan['wielo'] = None
            else:
                i += 1
            continue
        if w_str:
            if c == '\\':
                i += 2
            elif c == w_str:
                w_str = None
                i += 1
            else:
                typo_w_str = typo_w_str or c in TYPO
                i += 1
            continue
        if komentarz_od(s, i, ext):
            break
        if ext in KOMENTARZ_BLOKU and s.startswith('/*', i):
            stan['blok'] = True
            i += 2
            continue
        if ext in POTROJNY and (s.startswith('"""', i) or s.startswith("'''", i)):
            stan['wielo'] = s[i:i + 3]
            i += 3
            continue
        if c == '`' and ext in BACKTICK:
            stan['wielo'] = '`'
            i += 1
            continue
        if c in '"\'':
            w_str = c
            i += 1
            continue
        if c in TYPO:
            przed = s[:i].rstrip()
            if ext in ('sh', 'bash'):
                otw = bool(re.search(r'[A-Za-z_]\w*=$', przed))
            elif ext in TYLKO_MIESZANE:
                otw = False
            else:
                otw = przed.endswith(OTWIERAJACE) or bool(SLOWO_OTW.search(przed))
            zam = False
            if not otw and ext not in TYLKO_MIESZANE and ext not in BEZ_ZAMYKAJACEGO:
                j = next((k for k in range(i + 1, n) if s[k] in TYPO), None)
                reszta = s[j + 1:].lstrip() if j is not None else ''
                zam = bool(reszta) and reszta[0] in ZAMYKAJACE
            if otw or zam:
                return i + 1
        i += 1
    # `"tekst”` — string otwarty ASCII, niedomknięty do końca linii, w środku typograficzny:
    # to jest dokładnie ta pułapka („polski cudzysłów zamykający ucina string")
    if w_str and typo_w_str and ext in JEDNOLINIOWE and not s.rstrip().endswith('\\') \
            and not (ext in ('jsx', 'tsx') and w_str == "'"):
        return n
    return None


def cudzyslowy(root, pliki):
    for p in pliki:
        ext = rozszerzenie(p)
        if not ext:
            continue
        try:
            with open(os.path.join(root, p), 'rb') as f:
                dane = f.read()
        except OSError:
            continue
        if b'\0' in dane[:8192]:
            continue
        if not any(ch in dane.decode('utf-8', 'replace') for ch in TYPO):
            continue
        stan = {'blok': False, 'wielo': None}
        trafienia = []
        for nr, linia in enumerate(dane.decode('utf-8', 'replace').splitlines(), 1):
            if len(linia) > 4000:
                continue
            if typo_w_linii(linia, ext, stan) is not None:
                trafienia.append((nr, linia))
        for k, (nr, linia) in enumerate(trafienia[:20]):
            dalej = f' (+{len(trafienia) - 20} dalej w pliku)' if k == 19 and len(trafienia) > 20 else ''
            wypisz(plik=p, linia=nr, regula='cudzyslow-typograficzny', waga='blokuje',
                   opis=OPIS_TYPO + dalej, odcisk_tresci=odcisk(linia.strip()))


# --- Config -----------------------------------------------------------------------------

def sciezka_configu(root):
    if root:
        return os.path.join(root, 'ralph', 'config.md')
    # --sprawdz: runner woła z cwd = katalog modułu i bez RALPH_ROOT;
    # moduł leży w <projekt>/ralph-kontrole/lint-typy/
    tu = os.path.dirname(os.path.abspath(__file__))
    return os.path.normpath(os.path.join(tu, '..', '..', 'ralph', 'config.md'))


def wczytaj_testy(sciezka):
    """Pola sekcji ## Testy z ralph/config.md (bez komentarzy <!-- -->)."""
    try:
        with open(sciezka, encoding='utf-8') as f:
            tekst = f.read()
    except OSError:
        return {}
    m = re.search(r'^## Testy[ \t]*\n(.*?)(?=^## |\Z)', tekst, re.M | re.S)
    if not m:
        return {}
    sekcja = re.sub(r'<!--.*?-->', '', m.group(1), flags=re.S)
    return {k.strip(): v.strip() for k, v in re.findall(r'^- \*\*(.+?)\*\*:[ \t]*(.*?)[ \t]*$', sekcja, re.M)}


def komenda_z_pola(pola, nazwa):
    """Wartość pola albo None, gdy placeholder `[np. …]`, `brak`, puste."""
    w = (pola.get(nazwa) or '').strip()
    if not w or w.startswith('[') or w.lower() in ('brak', 'nie', '-', 'none'):
        return None
    return w


def komendy(pole):
    """Lista komend z pola. Projekt z kilkoma częściami wpisuje kilka komend, każdą ze swoim
    `cd` — tak ma Specky: `cd backend && ruff check app tests` · `cd frontend && npx eslint src`.
    Komendy w backtickach albo rozdzielone ` · ` / ` oraz ` / ` ; ` (spacje wokół średnika:
    sam `;` bywa częścią komendy)."""
    if not pole:
        return []
    w_backtickach = re.findall(r'`([^`]+)`', pole)
    if w_backtickach:
        return [k.strip() for k in w_backtickach if k.strip()]
    return [k.strip() for k in re.split(r'\s+·\s+|\s+oraz\s+|\s+;\s+', pole) if k.strip()]


def bez_sciezek(tokeny, cwd):
    """Tokeny bez ścieżek wpisanych w komendę (`ruff check app tests`), gdy hook podaje własną
    listę plików — inaczej linter przy commicie sprawdzałby cały katalog i blokował za cudze błędy.
    Ścieżka po opcji z wartością (`--config ruff.toml`) zostaje."""
    # Tylko ZA tokenem narzędzia: `python3 ../tools/fake_ruff.py check app` — skrypt narzędzia
    # sam jest ścieżką i musi zostać
    start = next((i for i, t in enumerate(tokeny) if any(n[0].search(t) for n in NARZEDZIA)), None)
    if start is None:
        return tokeny
    out = tokeny[:start + 1]
    for i in range(start + 1, len(tokeny)):
        t, poprzedni = tokeny[i], tokeny[i - 1]
        po_opcji = poprzedni.startswith('-') and '=' not in poprzedni
        if not t.startswith('-') and not po_opcji and os.path.exists(os.path.join(cwd, t)):
            continue
        out.append(t)
    return out


# --- Uruchamianie komend ---------------------------------------------------------------

def pozostalo(punkt):
    return max(1.0, BUDZET.get(punkt, 270) - (time.monotonic() - START))


def rozloz(cmd):
    """(katalog z `cd X &&` albo None, tokeny) — albo None, gdy komenda jest złożona
    (potok, kilka członów): wtedy nie da się dokleić plików ani flagi."""
    m = re.match(r'^\s*cd\s+(\S+)\s*&&\s*(.+)$', cmd, re.S)
    kat, reszta = (m.group(1), m.group(2)) if m else (None, cmd)
    if re.search(r'&&|\|\||[|;]|\$\(|`|\n', reszta):
        return None
    try:
        tokeny = shlex.split(reszta)
    except ValueError:
        return None
    return (kat, tokeny) if tokeny else None


def uruchom(tokeny, cwd, limit):
    """(proces, None) albo (None, 'brak' | 'czas')."""
    env = {**os.environ, 'NO_COLOR': '1', 'FORCE_COLOR': '0', 'CLICOLOR': '0', 'TERM': 'dumb',
           'PYTHONDONTWRITEBYTECODE': '1'}
    try:
        r = subprocess.run(tokeny, cwd=cwd, env=env, capture_output=True, text=True, timeout=limit)
    except subprocess.TimeoutExpired:
        return None, 'czas'
    except (FileNotFoundError, NotADirectoryError, PermissionError):
        return None, 'brak'
    except OSError:
        return None, 'brak'
    if r.returncode in (126, 127):
        return None, 'brak'
    return r, None


def wzgledna(root, cwd, p):
    """Ścieżka z wyjścia narzędzia (bezwzględna albo względem cwd) → względna od root.
    realpath po obu stronach: na macOS ruff zwraca /private/var/…, a root bywa /var/… ."""
    if not p:
        return p
    pelna = p if os.path.isabs(p) else os.path.join(cwd, p)
    return os.path.relpath(os.path.realpath(pelna), os.path.realpath(root))


_linie = {}


def tresc_linii(root, plik, nr):
    if plik not in _linie:
        try:
            with open(os.path.join(root, plik), encoding='utf-8', errors='replace') as f:
                _linie[plik] = f.read().splitlines()
        except OSError:
            _linie[plik] = []
    w = _linie[plik]
    return w[nr - 1].strip() if nr and 0 < nr <= len(w) else f'{plik}:{nr}'


def skrot_wyjscia(r):
    tekst = (r.stdout or '') + ('\n' + r.stderr if r.stderr else '')
    linie = [l.strip() for l in tekst.strip().splitlines() if l.strip()][:3]
    return ' | '.join(linie)[:300]


# --- Parsery wyjścia -----------------------------------------------------------------

KOD_BLOKUJE = re.compile(r'^(E9\d*|F\d*|B\d*|PLE|E0|F0)')
# `plik:linia[:kol][:] reszta` (ruff/flake8/mypy/go vet/pyright) albo `plik(l,c): reszta` (tsc)
LINIA_TEKST = re.compile(r'^\s*(?P<plik>(?:[A-Za-z]:)?[^\s:(][^:(\n]*?)'
                         r'(?:\((?P<l2>\d+),\d+\)|:(?P<l1>\d+)(?::\d+)?):?\s*-?\s*(?P<reszta>\S.*)$')


def _json(r):
    tekst = (r.stdout or '').strip()
    if not tekst:
        return None
    # narzędzie potrafi wypisać ostrzeżenie przed JSON-em — bierz od pierwszego nawiasu
    for start in ('[', '{'):
        k = tekst.find(start)
        if k >= 0:
            try:
                return json.loads(tekst[k:])
            except ValueError:
                continue
    return None


def waga_kodu(kod, severity=None):
    if severity in ('error', 'fatal', 2):
        return 'blokuje'
    if kod and KOD_BLOKUJE.match(kod):
        return 'blokuje'
    return 'ostrzega'


def parsuj_ruff(r):
    d = _json(r)
    if not isinstance(d, list):
        return None
    out = []
    for z in d:
        if not isinstance(z, dict):
            continue
        kod = z.get('code') or 'syntax-error'
        nr = (z.get('location') or {}).get('row')
        out.append((z.get('filename', ''), nr, kod, z.get('message', ''),
                    'blokuje' if kod == 'syntax-error' else waga_kodu(kod)))
    return out


def parsuj_pylint(r):
    d = _json(r)
    if not isinstance(d, list):
        return None
    return [(z.get('path', ''), z.get('line'), z.get('message-id') or z.get('symbol') or 'pylint',
             z.get('message', ''), 'blokuje' if z.get('type') in ('error', 'fatal') else 'ostrzega')
            for z in d if isinstance(z, dict)]


def parsuj_eslint(r):
    d = _json(r)
    if not isinstance(d, list):
        return None
    out = []
    for plik in d:
        for m in plik.get('messages') or []:
            kod = m.get('ruleId') or ('parse-error' if m.get('fatal') else 'eslint')
            out.append((plik.get('filePath', ''), m.get('line'), kod, m.get('message', ''),
                        'blokuje' if m.get('severity') == 2 or m.get('fatal') else 'ostrzega'))
    return out


def parsuj_biome(r):
    d = _json(r)
    if not isinstance(d, dict) or 'diagnostics' not in d:
        return None
    out = []
    for z in d['diagnostics'] or []:
        loc = z.get('location') or {}
        plik = (loc.get('path') or {}).get('file', '') if isinstance(loc.get('path'), dict) else loc.get('path', '')
        nr = None
        span, src = loc.get('span'), loc.get('sourceCode')
        if isinstance(span, list) and span and isinstance(src, str):
            nr = src[:span[0]].count('\n') + 1
        out.append((plik, nr, z.get('category') or 'biome', z.get('description', ''),
                    'blokuje' if z.get('severity') == 'error' else 'ostrzega'))
    return out


def parsuj_golangci(r):
    d = _json(r)
    if not isinstance(d, dict) or 'Issues' not in d:
        return None
    out = []
    for z in d.get('Issues') or []:
        pos = z.get('Pos') or {}
        out.append((pos.get('Filename', ''), pos.get('Line'), z.get('FromLinter') or 'golangci',
                    z.get('Text', ''), 'blokuje' if z.get('Severity') == 'error' else 'ostrzega'))
    return out


def parsuj_cargo(r):
    out, bylo = [], False
    for linia in (r.stdout or '').splitlines():
        try:
            z = json.loads(linia)
        except ValueError:
            continue
        if not isinstance(z, dict) or z.get('reason') != 'compiler-message':
            continue
        bylo = True
        m = z.get('message') or {}
        sp = (m.get('spans') or [{}])[0]
        kod = (m.get('code') or {}).get('code') or m.get('level') or 'cargo'
        out.append((sp.get('file_name', ''), sp.get('line_start'), kod, m.get('message', ''),
                    'blokuje' if m.get('level') == 'error' else 'ostrzega'))
    return out if bylo or (r.stdout or '').strip().startswith('{') else None


def parsuj_rubocop(r):
    d = _json(r)
    if not isinstance(d, dict) or 'files' not in d:
        return None
    out = []
    for plik in d.get('files') or []:
        for o in plik.get('offenses') or []:
            out.append((plik.get('path', ''), (o.get('location') or {}).get('line'), o.get('cop_name') or 'rubocop',
                        o.get('message', ''), 'blokuje' if o.get('severity') in ('error', 'fatal') else 'ostrzega'))
    return out


def parsuj_shellcheck(r):
    d = _json(r)
    if not isinstance(d, list):
        return None
    return [(z.get('file', ''), z.get('line'), f'SC{z.get("code")}', z.get('message', ''),
             'blokuje' if z.get('level') == 'error' else 'ostrzega') for z in d if isinstance(z, dict)]


def parsuj_pyright(r):
    d = _json(r)
    if not isinstance(d, dict) or 'generalDiagnostics' not in d:
        return None
    out = []
    for z in d.get('generalDiagnostics') or []:
        if z.get('severity') not in ('error', 'warning'):
            continue
        out.append((z.get('file', ''), ((z.get('range') or {}).get('start') or {}).get('line', -1) + 1,
                    z.get('rule') or 'pyright', z.get('message', ''),
                    'blokuje' if z.get('severity') == 'error' else 'ostrzega'))
    return out


def parsuj_tekst(r, cwd, typy=False):
    """`plik:linia: …` i `plik(l,c): …` — flake8, ruff bez JSON, go vet, mypy, tsc, pyright."""
    out = []
    for linia in ((r.stdout or '') + '\n' + (r.stderr or '')).splitlines():
        m = LINIA_TEKST.match(linia)
        if not m:
            continue
        plik = m.group('plik').strip()
        if not os.path.isfile(plik if os.path.isabs(plik) else os.path.join(cwd, plik)):
            continue
        nr = int(m.group('l1') or m.group('l2'))
        reszta = m.group('reszta').strip()
        if re.match(r'note:', reszta):
            continue
        poziom = None
        pm = re.match(r'(error|warning)\b:?\s*', reszta)
        if pm:
            poziom = pm.group(1)
            reszta = reszta[pm.end():]
        kod = None
        km = re.match(r'(TS\d+)\s*:\s*', reszta)             # tsc: error TS2322: …
        if km:
            kod, reszta = km.group(1), reszta[km.end():]
        else:
            km = re.search(r'\s+\[([\w.-]+)\]\s*$', reszta)    # mypy: …  [assignment]
            if km:
                kod, reszta = km.group(1), reszta[:km.start()]
            else:
                km = re.match(r'([A-Z]{1,4}\d{2,4})\s+', reszta)   # flake8/ruff: E501 …
                if km:
                    kod, reszta = km.group(1), reszta[km.end():]
                else:
                    km = re.search(r'\((report\w+)\)\s*$', reszta)   # pyright tekstowo
                    if km:
                        kod, reszta = km.group(1), reszta[:km.start()]
        if typy:
            if poziom is None and kod is None:
                continue
            waga = 'blokuje' if poziom in ('error', None) else 'ostrzega'
            kod = kod or 'error'
        else:
            waga = 'blokuje' if poziom == 'error' else waga_kodu(kod)
            kod = kod or ('vet' if 'vet' in (r.args[0:2] if isinstance(r.args, list) else []) else 'linter')
        out.append((plik, nr, kod, reszta.strip(), waga))
    return out


def parsuj_format(r):
    """black / isort / prettier / cargo fmt / ruff format: plik do przeformatowania → ostrzega."""
    out = []
    for linia in ((r.stdout or '') + '\n' + (r.stderr or '')).splitlines():
        m = (re.match(r'^would reformat (.+)$', linia.strip())                      # black, ruff format
             or re.match(r'^ERROR: (.+?) Imports are incorrectly sorted', linia.strip())   # isort
             or re.match(r'^\[warn\] (\S.*)$', linia.strip())                        # prettier
             or re.match(r'^Diff in (.+?) at line', linia.strip()))                  # cargo fmt
        if m:
            out.append((m.group(1).strip(), None, 'format', 'plik wymaga formatowania — uruchom formatter', 'ostrzega'))
    return out


PARSERY = {'ruff': parsuj_ruff, 'pylint': parsuj_pylint, 'eslint': parsuj_eslint, 'biome': parsuj_biome,
           'golangci': parsuj_golangci, 'cargo': parsuj_cargo, 'rubocop': parsuj_rubocop,
           'shellcheck': parsuj_shellcheck, 'format': parsuj_format}


# --- Linter ---------------------------------------------------------------------------

def narzedzie(cmd):
    for wz, exts, z_lista, flaga, parser in NARZEDZIA:
        if wz.search(cmd):
            if parser == 'ruff' and re.search(r'\bruff\s+format\b', cmd):
                return exts, z_lista, None, 'format'
            return exts, z_lista, flaga, parser
    return None


def przygotuj(tokeny, flaga, parser):
    """Zdejmuje `--fix`, dokłada `--check` formatterowi i flagę JSON (przed `--`, gdy jest)."""
    tokeny = [t for t in tokeny if t not in ('--fix', '--fix-only', '--unsafe-fixes', '-w', '--write')]
    if parser == 'format':
        for slowo, check in FORMATTER_CHECK.items():
            if slowo in tokeny and not any(t in ('--check', '--check-only', '-c') for t in tokeny):
                tokeny.append(check)
                break
    if flaga and not any('json' in t for t in tokeny):
        k = tokeny.index('--') if '--' in tokeny else len(tokeny)
        tokeny = tokeny[:k] + flaga + tokeny[k:]
    return tokeny


def emituj(root, cwd, wyniki, prefiks):
    ile = 0
    widziane = set()
    for plik, nr, kod, opis, waga in wyniki:
        plik = wzgledna(root, cwd, plik)
        klucz = (plik, nr, kod)
        if klucz in widziane:
            continue
        widziane.add(klucz)
        ile += 1
        if ile > MAX_ZNALEZISK:
            wypisz(plik=plik, linia=None, regula=f'{prefiks}-wiecej', waga='ostrzega', odcisk_pliku=False,
                   opis=f'… i {len(wyniki) - MAX_ZNALEZISK} kolejnych zgłoszeń narzędzia', odcisk_tresci=prefiks)
            break
        wypisz(plik=plik, linia=nr, regula=f'{prefiks}/{kod}', waga=waga, opis=(opis or kod)[:300],
               odcisk_tresci=odcisk(tresc_linii(root, plik, nr) if nr else f'{plik}:{kod}'))


def linter(root, punkt, cmd, pliki):
    narz = narzedzie(cmd)
    pierwszy = pliki[0] if pliki else 'ralph/config.md'
    roz = rozloz(cmd)
    if narz is None or roz is None:
        # Nieznane narzędzie: nie wiadomo, które pliki mu podać ani jak czytać wynik.
        # Przy fazie uruchamiamy je tak, jak stoi; przy commicie jedno ostrzeżenie.
        if punkt == 'commit':
            wypisz(plik=pierwszy, linia=None, regula='linter-nieznany', waga='ostrzega', odcisk_pliku=False,
                   opis=f'nieznane narzędzie w „Linter/formatter" ({cmd[:60]}) — przy commicie pominięte; '
                        f'uruchamiane w całości przy fazie / CI. Znane: ruff, flake8, pylint, black, isort, '
                        f'eslint, biome, prettier, golangci-lint, go vet, cargo clippy/fmt, rubocop, shellcheck',
                   odcisk_tresci='linter-nieznany')
            return
        if not pliki:
            return
        tokeny, cwd, parser = ['bash', '-c', cmd], root, 'tekst'
    else:
        exts, z_lista, flaga, parser = narz
        kat, tokeny = roz
        cwd = os.path.normpath(os.path.join(root, kat)) if kat else root
        pasujace = [p for p in pliki if rozszerzenie_dowolne(p) in exts]
        if not pasujace:
            return
        tokeny = przygotuj(tokeny, flaga, parser)
        if z_lista:
            # Zakres (zmienione / całość) liczy runner — moduł dostaje gotową listę. Ścieżki
            # względem katalogu, w którym narzędzie zostanie uruchomione (`cd backend && ruff check`).
            lista = [os.path.relpath(os.path.join(root, p), cwd) for p in pasujace]
            lista = [p for p in lista if not p.startswith('..')]
            if not lista:
                return
            if len(lista) <= MAX_LISTA:
                tokeny = bez_sciezek(tokeny, cwd)
                k = tokeny.index('--') if '--' in tokeny else len(tokeny)
                tokeny = tokeny[:k] + lista + tokeny[k:]
    limit = min(LIMIT_COMMIT if punkt == 'commit' else LIMIT_PELNY, pozostalo(punkt))
    r, blad = uruchom(tokeny, cwd, limit)
    if blad == 'brak':
        wypisz(plik=pierwszy, linia=None, regula='linter-niedostepny', waga='ostrzega', odcisk_pliku=False,
               opis=f'narzędzie z „Linter/formatter" nie jest dostępne ({tokeny[0]}) — operacja przeszła bez '
                    f'lintera; zainstaluj je albo popraw pole w ralph/config.md',
               odcisk_tresci='linter-niedostepny:' + tokeny[0])
        return
    if blad == 'czas':
        wypisz(plik=pierwszy, linia=None, regula='linter-limit-czasu', waga='ostrzega', odcisk_pliku=False,
               opis=f'linter nie zmieścił się w {int(limit)} s — operacja przeszła bez niego; zawęź komendę',
               odcisk_tresci='linter-limit-czasu')
        return
    wyniki = PARSERY[parser](r) if parser in PARSERY else None
    if wyniki is None:
        wyniki = parsuj_tekst(r, cwd)
    if wyniki:
        emituj(root, cwd, wyniki, 'linter')
    elif r.returncode != 0:
        wypisz(plik=pierwszy, linia=None, regula='linter-zglosil', waga='ostrzega', odcisk_pliku=False,
               opis=f'linter zakończył się kodem {r.returncode}, wynik nie dał się sparsować: {skrot_wyjscia(r)}',
               odcisk_tresci='linter-zglosil:' + skrot_wyjscia(r))


def rozszerzenie_dowolne(p):
    return p.rsplit('.', 1)[-1].lower() if '.' in os.path.basename(p) else ''


# --- Typechecker ------------------------------------------------------------------------

def typy(root, punkt, cmd, pierwszy):
    roz = rozloz(cmd)
    if roz is None:
        tokeny, cwd = ['bash', '-c', cmd], root
    else:
        kat, tokeny = roz
        cwd = os.path.normpath(os.path.join(root, kat)) if kat else root
        if re.search(r'\bpyright\b', cmd) and not any('json' in t for t in tokeny):
            tokeny = tokeny + ['--outputjson']
    limit = min(LIMIT_PELNY, pozostalo(punkt))
    r, blad = uruchom(tokeny, cwd, limit)
    if blad == 'brak':
        wypisz(plik=pierwszy, linia=None, regula='typy-niedostepne', waga='ostrzega', odcisk_pliku=False,
               opis=f'typechecker z „Komenda typów" nie jest dostępny ({tokeny[0]}) — operacja przeszła bez niego',
               odcisk_tresci='typy-niedostepne:' + tokeny[0])
        return
    if blad == 'czas':
        wypisz(plik=pierwszy, linia=None, regula='typy-limit-czasu', waga='ostrzega', odcisk_pliku=False,
               opis=f'typechecker nie zmieścił się w {int(limit)} s — operacja przeszła bez niego',
               odcisk_tresci='typy-limit-czasu')
        return
    wyniki = parsuj_pyright(r) if re.search(r'\bpyright\b', cmd) else None
    if wyniki is None:
        wyniki = parsuj_tekst(r, cwd, typy=True)
    if wyniki:
        emituj(root, cwd, wyniki, 'typy')
    elif r.returncode != 0:
        wypisz(plik=pierwszy, linia=None, regula='typy-zglosily', waga='ostrzega', odcisk_pliku=False,
               opis=f'typechecker zakończył się kodem {r.returncode}, wynik nie dał się sparsować: {skrot_wyjscia(r)}',
               odcisk_tresci='typy-zglosily:' + skrot_wyjscia(r))


# --- Główne ------------------------------------------------------------------------------

def sprawdz():
    pola = wczytaj_testy(sciezka_configu(os.environ.get('RALPH_ROOT')))
    lint = komenda_z_pola(pola, 'Linter/formatter')
    typ = komenda_z_pola(pola, 'Komenda typów')
    if not lint and not typ:
        print('uwaga: lint-typy: brak lintera i typecheckera w ## Testy — działa tylko reguła cudzysłowu; '
              'wpisz np. "ruff check" / "npx eslint ." i "mypy src" / "npx tsc --noEmit"')
        return 0
    uwagi = []
    for nazwa, cmd in [('linter', k) for k in komendy(lint)] + [('typechecker', k) for k in komendy(typ)]:
        roz = rozloz(cmd)
        if roz is None:
            uwagi.append(f'{nazwa} „{cmd[:40]}" to komenda złożona — bez listy plików i parsowania')
            continue
        prog = roz[1][0]
        if '/' not in prog and not shutil.which(prog):
            uwagi.append(f'{nazwa}: „{prog}" nie jest w PATH')
        if nazwa == 'linter' and narzedzie(cmd) is None:
            uwagi.append(f'linter „{cmd[:40]}" nieznany — przy commicie pominięty, uruchamiany tylko przy fazie / CI')
    if uwagi:
        print('uwaga: lint-typy: ' + '; '.join(uwagi))
    else:
        print(f'ok: linter: {lint or "brak"}; typy: {typ or "brak"}')
    return 0


def main():
    if '--sprawdz' in sys.argv:
        return sprawdz()
    punkt = os.environ.get('RALPH_PUNKT', 'commit')
    root = os.environ.get('RALPH_ROOT') or os.getcwd()
    pliki = [l.strip() for l in sys.stdin.read().splitlines() if l.strip()]
    cudzyslowy(root, pliki)
    pola = wczytaj_testy(sciezka_configu(root))
    lint = komenda_z_pola(pola, 'Linter/formatter')
    typ = komenda_z_pola(pola, 'Komenda typów')
    # Każda komenda osobno: `cd backend && ruff …` dostaje tylko pliki z backend/, eslint z frontend/
    if pliki:
        for cmd in komendy(lint):
            linter(root, punkt, cmd, pliki)
    # tsc / mypy nie umieją sensownie „tylko te pliki", a na dużym projekcie nie mieszczą
    # się w budżecie commita — pełny przebieg przy fazie / CI
    if punkt in ('faza', 'ci'):
        for cmd in komendy(typ):
            typy(root, punkt, cmd, pliki[0] if pliki else 'ralph/config.md')
    return 0


if __name__ == '__main__':
    sys.exit(main())
