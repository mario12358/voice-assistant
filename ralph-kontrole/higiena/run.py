#!/usr/bin/env python3
"""Kontrola `higiena` — lista plików na stdin, znaleziska JSON w liniach na stdout.

Rzeczy, które nie powinny trafić do repozytorium: katalogi budowania, plik po 5 MB,
manifest zmieniony bez lockfile'a, wersje pływające, binaria w katalogach kodu,
zminifikowany bundle, pliki IDE, artefakty wizualne przy wyłączonym commicie,
brak .gitignore.

Runner odfiltrowuje ze stdin pliki > 1 MB, lockfile'y i `*.min.*`, a przy
`git add x && git commit` plik x nie jest jeszcze w indeksie — więc lista do reguł
po ścieżce i rozmiarze jest składana z trzech źródeł: stdin ∪ indeks ∪ zmiany
względem HEAD (przy ci: diff od podstawy gałęzi). Tylko tak 6 MB zrzut bazy
dodany chwilę wcześniej przez `git add` ma szansę zostać zauważony.
"""
import json
import os
import re
import subprocess
import sys

PLIK_TESTU = re.compile(r'(^|/)(tests?|__tests__|spec|e2e)/|(^|/)test_[^/]*$|[._-](test|spec)\.[^/]+$|_test\.[^/]+$')
KATALOG_BUDOWANIA = re.compile(r'(^|/)(node_modules|dist|build|out|\.venv|venv|__pycache__|coverage|htmlcov|'
                               r'\.next|\.nuxt|\.pytest_cache|\.mypy_cache|\.ruff_cache)/')
# `target/` to katalog budowania tylko w Ruście/Javie — w innych projektach bywa zwykłym katalogiem
TARGET = re.compile(r'(^|/)target/')
MANIFEST_TARGET = ('Cargo.toml', 'pom.xml', 'build.gradle', 'build.gradle.kts')
SMIECI = [('*.pyc', re.compile(r'\.pyc$')), ('*.pyo', re.compile(r'\.pyo$')),
          ('.DS_Store', re.compile(r'(^|/)\.DS_Store$'))]
KATALOG_KODU = re.compile(r'(^|/)(src|app|lib|backend|frontend|pkg|cmd)/')
ZASOB = re.compile(r'\.(png|jpe?g|gif|webp|ico|svg|woff2?|ttf|otf|wasm)$|(^|/)bun\.lockb$')
PDF_W_DOCS = re.compile(r'(^|/)docs/.*\.pdf$')
BUNDLE = re.compile(r'\.(m?js|cjs|css)$')
ZMINIFIKOWANY_Z_NAZWY = re.compile(r'\.min\.[^/]+$')
VENDOR = re.compile(r'(^|/)vendor/')
MAX_LINIA_BUNDLE = 5000
IDEA = re.compile(r'(^|/)\.idea/')
VSCODE = re.compile(r'(^|/)\.vscode/([^/]+)$')
VSCODE_DOZWOLONE = {'extensions.json', 'launch.json', 'tasks.json', 'settings.json'}
SCIEZKA_BEZWZGLEDNA = re.compile(r'/Users/|/home/|[A-Za-z]:\\')
IDE_LUZNY = re.compile(r'\.sw[po]$|~$|(^|/)Thumbs\.db$')
KOD = re.compile(r'\.(py|js|jsx|ts|tsx|mjs|cjs|java|kt|go|rb|php|rs|cs|swift|scala|sh|sql|vue|svelte)$')
# manifest → lockfile'e w tym samym katalogu, w kolejności szukania
LOCKFILE = {
    'package.json': ['package-lock.json', 'pnpm-lock.yaml', 'yarn.lock', 'bun.lockb', 'bun.lock'],
    'pyproject.toml': ['poetry.lock', 'uv.lock', 'pdm.lock'],
    'Cargo.toml': ['Cargo.lock'],
    'go.mod': ['go.sum'],
    'Gemfile': ['Gemfile.lock'],
    'composer.json': ['composer.lock'],
    'deno.json': ['deno.lock'],
    'deno.jsonc': ['deno.lock'],
}
REQUIREMENTS = re.compile(r'(^|/)requirements[^/]*\.txt$')
KLUCZE_NPM = ('dependencies', 'devDependencies', 'peerDependencies', 'optionalDependencies')
GIT_BEZ_REF = re.compile(r'^(git\+|git://|github:|gitlab:|bitbucket:|https?://(github|gitlab|bitbucket)\.(com|org)/)')
# sekcje TOML, których zmiana wymaga przebudowy lockfile'a (pyproject + Cargo)
SEKCJA_ZALEZNOSCI_TOML = re.compile(
    r'^(project|project\.optional-dependencies|dependency-groups|tool\.poetry\.dependencies|'
    r'tool\.poetry\.dev-dependencies|tool\.poetry\.group\.[^.\]]+\.dependencies|tool\.uv\.sources|'
    r'dependencies|dev-dependencies|build-dependencies|workspace\.dependencies|target\..+\.dependencies|'
    r'(dev-|build-)?dependencies\.[^\]]+)$')
# sekcje TOML z zależnościami PRODUKCYJNYMI — tylko tam wersja pływająca ostrzega
SEKCJA_PRODUKCYJNA_TOML = re.compile(r'^(tool\.poetry\.dependencies|dependencies|dependencies\.[^\]]+)$')


def wypisz(**z):
    print(json.dumps(z, ensure_ascii=False))


def git(root, *args):
    try:
        r = subprocess.run(['git', '-C', root, *args], capture_output=True, text=True, timeout=20)
    except (OSError, subprocess.TimeoutExpired):
        return None
    return r.stdout if r.returncode == 0 else None


def linie(s):
    return [x for x in (s or '').splitlines() if x.strip()]


def czytaj(root, p, ile=None):
    try:
        with open(os.path.join(root, p), 'rb') as f:
            return f.read(ile) if ile else f.read()
    except OSError:
        return None


def tekst(root, p):
    dane = czytaj(root, p)
    return None if dane is None or b'\0' in dane[:8192] else dane.decode('utf-8', 'replace')


def lista_nazw(nazwy, max_=5):
    return ', '.join(nazwy[:max_]) + (f' (+{len(nazwy) - max_})' if len(nazwy) > max_ else '')


class Zakres:
    """Co faktycznie wchodzi do commita / zakresu CI — ponad to, co dał runner."""

    def __init__(self, root, punkt):
        self.root, self.punkt = root, punkt
        self.indeks = set(linie(git(root, 'diff', '--cached', '--name-only', '--diff-filter=ACMR')))
        self.vs_head = set(linie(git(root, 'diff', '--name-only', '--diff-filter=ACMR', 'HEAD')))
        self.baza = self.baza_ci() if punkt == 'ci' else None
        self.zakres_ci = set(linie(git(root, 'diff', '--name-only', '--diff-filter=ACMR',
                                       f'{self.baza}...HEAD'))) if self.baza else set()

    def baza_ci(self):
        od = os.environ.get('RALPH_OD')
        if od:
            return od
        head = (git(self.root, 'rev-parse', 'HEAD') or '').strip()
        for ref in ('origin/HEAD', 'origin/main', 'origin/master', 'main', 'master'):
            mb = (git(self.root, 'merge-base', 'HEAD', ref) or '').strip()
            if mb and mb != head:
                return mb
        return 'HEAD~1' if git(self.root, 'rev-parse', '--verify', '-q', 'HEAD~1') else None

    def dodatkowe(self):
        return self.indeks | self.vs_head | self.zakres_ci

    def zmieniony(self, p):
        """Czy plik ma JAKĄKOLWIEK zmianę w zakresie — przy commicie także niezaindeksowaną:
        `git add package.json package-lock.json && git commit` w chwili hooka ma oba
        poza indeksem, a runner lockfile'a ze stdin wyciął; zmiana w drzewie to jedyny ślad."""
        return p in (self.zakres_ci if self.punkt == 'ci' else self.vs_head | self.indeks)

    def sledzony(self, p):
        return bool(linie(git(self.root, 'ls-files', '--', p)))

    def poprzednia_wersja(self, p):
        ref = self.baza if self.punkt == 'ci' else 'HEAD'
        return git(self.root, 'show', f'{ref}:{p}') if ref else None


# --- katalog-budowania / plik-za-duzy ----------------------------------------------

def katalog_budowania(p, root):
    """Klucz grupy (ścieżka katalogu albo wzorzec pliku) albo None."""
    m = KATALOG_BUDOWANIA.search(p)
    if m:
        return p[:m.end() - 1]
    m = TARGET.search(p)
    if m:
        nad = os.path.join(root, p[:m.start()].rstrip('/') if m.start() else '')
        if any(os.path.isfile(os.path.join(nad, mf)) for mf in MANIFEST_TARGET):
            return p[:m.end() - 1]
    for klucz, wz in SMIECI:
        if wz.search(p):
            return klucz
    return None


def grupami(grupy, regula, waga, opis):
    """Jedno znalezisko na katalog: pierwszy plik + licznik. Odcisk = klucz grupy, bez pliku —
    wyjątek „ten katalog jest celowo w repo" ma przeżyć zmianę pierwszego pliku w grupie."""
    for klucz, pliki in sorted(grupy.items()):
        dalej = f' (+{len(pliki) - 1} dalej w {klucz})' if len(pliki) > 1 else ''
        wypisz(plik=pliki[0], linia=None, regula=regula, waga=waga, odcisk_pliku=False,
               opis=opis(klucz) + dalej, odcisk_tresci=klucz)


def katalogi_budowania(root, pliki):
    grupy = {}
    for p in pliki:
        klucz = katalog_budowania(p, root)
        if klucz:
            grupy.setdefault(klucz, []).append(p)
    grupami(grupy, 'katalog-budowania', 'blokuje',
            lambda k: f'wynik budowania / cache ({k}) w commicie — `git rm -r --cached {k}` i dopisz do .gitignore')


def rozmiary(root, pliki, artefakty_dir, commit_artefaktow):
    limit_mb = float(os.environ.get('RALPH_HIGIENA_MAX_MB') or 5)
    for p in pliki:
        try:
            mb = os.path.getsize(os.path.join(root, p)) / 1_000_000
        except OSError:
            continue
        if mb <= limit_mb:
            continue
        # Nagranie z artifacts/flows/ przy `Commit artefakty: tak` to świadoma decyzja (sekcja 0.5):
        # nie blokujemy, przypominamy o LFS od 50 MB
        if commit_artefaktow and artefakty_dir and p.startswith(artefakty_dir):
            if mb > 50:
                wypisz(plik=p, linia=None, regula='plik-za-duzy', waga='ostrzega',
                       opis=f'artefakt {mb:.1f} MB — powyżej 50 MB rozważ Git LFS (sekcja 0.5)', odcisk_tresci=p)
            continue
        wypisz(plik=p, linia=None, regula='plik-za-duzy', waga='blokuje',
               opis=f'plik {mb:.1f} MB (limit {limit_mb:g} MB) — nie commituj: dane poza repo albo Git LFS, '
                    f'dopisz do .gitignore (limit: RALPH_HIGIENA_MAX_MB)', odcisk_tresci=p)


# --- lockfile-nieaktualny ------------------------------------------------------------

def json_bez_komentarzy(t):
    try:
        return json.loads(t)
    except ValueError:
        try:
            return json.loads(re.sub(r'^\s*//.*$', '', t, flags=re.M))      # deno.jsonc
        except ValueError:
            return None


def linie_toml(t):
    """[(sekcja, linia logiczna)] — tablice wieloliniowe sklejone, komentarze i puste pominięte."""
    out, sekcja, bufor, glebokosc = [], '', '', 0
    for linia in t.splitlines():
        s = re.sub(r'(^|\s)#.*$', '', linia).strip()
        if not s:
            continue
        if not bufor and s.startswith('['):
            m = re.match(r'^\[\[?\s*([^\]]+?)\s*\]\]?', s)
            sekcja = m.group(1).strip() if m else s
            continue
        bufor = (bufor + ' ' + s).strip() if bufor else s
        glebokosc += s.count('[') - s.count(']') + s.count('{') - s.count('}')
        if glebokosc <= 0:
            out.append((sekcja, bufor))
            bufor, glebokosc = '', 0
    if bufor:
        out.append((sekcja, bufor))
    return out


def zaleznosci_manifestu(nazwa, t):
    """Zbiór wpisów sekcji zależności — równość starego i nowego zbioru znaczy, że zmiana
    manifestu (wersja projektu, skrypty) lockfile'a nie potrzebuje. None = nie umiem czytać."""
    if nazwa in ('package.json', 'composer.json', 'deno.json', 'deno.jsonc'):
        d = json_bez_komentarzy(t)
        if not isinstance(d, dict):
            return None
        klucze = {'package.json': KLUCZE_NPM, 'composer.json': ('require', 'require-dev')}.get(nazwa, ('imports', 'scopes'))
        out = set()
        for k in klucze:
            blok = d.get(k) or {}
            out |= {f'{k}:{n}={json.dumps(w, sort_keys=True)}' for n, w in (blok.items() if isinstance(blok, dict) else [])}
        return out
    if nazwa in ('pyproject.toml', 'Cargo.toml'):
        out = set()
        for sekcja, linia in linie_toml(t):
            if not SEKCJA_ZALEZNOSCI_TOML.match(sekcja):
                continue
            if sekcja == 'project' and not re.match(r'^dependencies\s*=', linia):
                continue
            out.add(f'{sekcja}|{linia}')
        return out
    if nazwa == 'go.mod':
        out, w_bloku = set(), None
        for linia in t.splitlines():
            s = linia.split('//')[0].strip()
            if not s:
                continue
            if re.match(r'^(require|replace|exclude)\s*\($', s):
                w_bloku = s.split()[0]
            elif s == ')':
                w_bloku = None
            elif w_bloku:
                out.add(f'{w_bloku} {s}')
            elif re.match(r'^(require|replace|exclude)\s', s):
                out.add(s)
        return out
    if nazwa == 'Gemfile':
        return {s.strip() for s in t.splitlines() if re.match(r'^\s*(gem|gemspec|git|github|path)\b', s)}
    return None


def lockfile_nieaktualny(root, pliki, zakres):
    for p in pliki:
        nazwa = os.path.basename(p)
        if nazwa not in LOCKFILE:
            continue
        katalog = os.path.dirname(p)
        lock = next((os.path.join(katalog, l) if katalog else l for l in LOCKFILE[nazwa]
                     if zakres.sledzony(os.path.join(katalog, l) if katalog else l)), None)
        if not lock:
            continue                # projekt nie trzyma lockfile'a — nie nasza sprawa
        if zakres.zmieniony(lock):
            continue
        nowy = tekst(root, p)
        if nowy is None:
            continue
        nowe = zaleznosci_manifestu(nazwa, nowy)
        if nowe is None:
            continue
        stary = zakres.poprzednia_wersja(p)
        stare = zaleznosci_manifestu(nazwa, stary) if stary is not None else set()
        if stare is None or stare == nowe:
            continue
        wypisz(plik=p, linia=None, regula='lockfile-nieaktualny', waga='blokuje',
               opis=f'zależności w {nazwa} zmienione, a {lock} nie — uruchom instalację '
                    f'(npm install / poetry lock / cargo update …) i dodaj {lock} do tego samego commita',
               odcisk_tresci=f'{p}|{lock}')


# --- wersja-plywajaca ----------------------------------------------------------------

def plywa_npm(wersja):
    w = str(wersja).strip()
    if w in ('', '*', 'latest', 'x', 'X'):
        return True
    if re.match(r'^>=?\s*[\w.*-]+$', w):                       # >=1.2 bez `,<` / `<`
        return True
    return bool(GIT_BEZ_REF.match(w) and '#' not in w)


def plywa_pip(spec):
    """Nazwa pakietu, gdy specyfikator PEP 508 nie ma górnej granicy; None gdy ma albo to nie pakiet."""
    s = spec.split(';')[0].split(' #')[0].strip().strip('"\'')
    if not s or s.startswith(('-', '#')):
        return None
    m = re.match(r'^([A-Za-z0-9][\w.-]*)', s)
    if not m:
        return None
    nazwa = m.group(1)
    if '@' in s and '://' in s:                                 # pkg @ url — pływa tylko git bez refa
        url = s.split('@', 1)[1].strip()
        return nazwa if 'git+' in url and not re.search(r'\.git@[^/]+$|@[0-9a-f]{7,40}$|@v?\d[\w.]*$|#', url) else None
    return None if re.search(r'===|==|~=|<', s) else nazwa


def plywa_toml(wartosc):
    """Wartość zależności w pyproject (poetry) / Cargo: string albo tabela inline."""
    w = wartosc.strip()
    if w.startswith('{'):
        g = re.search(r'\bgit\s*=', w)
        if g and not re.search(r'\b(rev|tag|branch)\s*=', w):
            return True
        v = re.search(r'\bversion\s*=\s*["\']([^"\']*)["\']', w)
        return bool(v) and plywa_toml(f'"{v.group(1)}"')
    if w.startswith('['):                                        # poetry: lista wariantów
        return any(plywa_toml(x) for x in re.findall(r'\{[^}]*\}', w))
    w = w.strip('"\'')
    if w in ('', '*', 'latest'):
        return True
    return bool(re.match(r'^>=?\s*[\w.*-]+$', w))


def wersje_plywajace(root, pliki):
    for p in pliki:
        nazwa = os.path.basename(p)
        plywaja = []
        if nazwa == 'package.json':
            d = json_bez_komentarzy(tekst(root, p) or '')
            deps = (d or {}).get('dependencies') if isinstance(d, dict) else None
            plywaja = [n for n, w in (deps or {}).items() if isinstance(w, str) and plywa_npm(w)] \
                if isinstance(deps, dict) else []
        elif REQUIREMENTS.search(p):
            plywaja = [n for n in (plywa_pip(l) for l in (tekst(root, p) or '').splitlines()) if n]
        elif nazwa in ('pyproject.toml', 'Cargo.toml'):
            for sekcja, linia in linie_toml(tekst(root, p) or ''):
                if sekcja == 'project' and re.match(r'^dependencies\s*=', linia):
                    plywaja += [n for n in (plywa_pip(x) for x in re.findall(r'"([^"]*)"|\'([^\']*)\'', linia)
                                            for x in x if x) if n]
                elif SEKCJA_PRODUKCYJNA_TOML.match(sekcja) and not sekcja.startswith('dependencies.'):
                    m = re.match(r'^([\w.-]+|"[^"]+")\s*=\s*(.+)$', linia)
                    if m and m.group(1).strip('"') != 'python' and plywa_toml(m.group(2)):
                        plywaja.append(m.group(1).strip('"'))
            # Cargo: [dependencies.nazwa] jako osobna tabela — pływa, gdy version bez granicy
            # albo git bez rev/tag/branch
            tabele = {}
            for sekcja, linia in linie_toml(tekst(root, p) or ''):
                m = re.match(r'^dependencies\.([^\]]+)$', sekcja)
                if m:
                    tabele.setdefault(m.group(1).strip('"'), []).append(linia)
            for n, ls in tabele.items():
                if any(re.match(r'^(rev|tag|branch)\s*=', l) for l in ls):
                    continue
                v = next((l for l in ls if re.match(r'^version\s*=', l)), None)
                if (v and plywa_toml(v.split('=', 1)[1])) or (not v and any(re.match(r'^git\s*=', l) for l in ls)):
                    plywaja.append(n)
        else:
            continue
        if plywaja:
            wypisz(plik=p, linia=None, regula='wersja-plywajaca', waga='ostrzega',
                   opis=f'zależności produkcyjne bez górnej granicy wersji: {lista_nazw(plywaja)} — '
                        f'przypnij wersję (lockfile nie pomoże przy świeżej instalacji z manifestu)',
                   odcisk_tresci=f'{p}|' + ','.join(sorted(plywaja)))


# --- plik-binarny / bundle-zminifikowany ----------------------------------------------

def binaria(root, pliki):
    for p in pliki:
        if not KATALOG_KODU.search(p) or ZASOB.search(p) or PDF_W_DOCS.search(p) or PLIK_TESTU.search(p):
            continue
        if katalog_budowania(p, root):
            continue
        dane = czytaj(root, p, 8192)
        if dane and b'\0' in dane:
            wypisz(plik=p, linia=None, regula='plik-binarny', waga='ostrzega',
                   opis='plik binarny w katalogu kodu — nie jest obrazem ani fontem; jeśli to dane/model, '
                        'trzymaj poza repo albo w LFS', odcisk_tresci=p)


def bundle(root, pliki):
    for p in pliki:
        if not BUNDLE.search(p) or VENDOR.search(p) or PLIK_TESTU.search(p) or katalog_budowania(p, root):
            continue
        if ZMINIFIKOWANY_Z_NAZWY.search(os.path.basename(p)):
            powod = 'nazwa *.min.*'
        else:
            dane = czytaj(root, p, 1_000_000) or b''
            if b'\0' in dane[:8192] or max((len(l) for l in dane.split(b'\n')), default=0) <= MAX_LINIA_BUNDLE:
                continue
            powod = f'linia > {MAX_LINIA_BUNDLE} znaków'
        wypisz(plik=p, linia=None, regula='bundle-zminifikowany', waga='ostrzega',
               opis=f'zminifikowany bundle ({powod}) — wynik budowania należy do dist/ i .gitignore; '
                    f'biblioteka zewnętrzna → vendor/', odcisk_tresci=p)


# --- plik-ide / artefakty-wizualne / brak-gitignore -----------------------------------

def pliki_ide(root, pliki):
    grupy = {}
    for p in pliki:
        m = IDEA.search(p)
        if m:
            grupy.setdefault(p[:m.end() - 1], []).append(p)
            continue
        m = VSCODE.search(p)
        if m:
            if m.group(2) not in VSCODE_DOZWOLONE:
                wypisz(plik=p, linia=None, regula='plik-ide', waga='ostrzega',
                       opis='plik .vscode/ spoza wspólnych (extensions/launch/tasks/settings) — ustawienie osobiste, '
                            'dopisz do .gitignore', odcisk_tresci=p)
                continue
            t = tekst(root, p) or ''
            for nr, linia in enumerate(t.splitlines(), 1):
                if SCIEZKA_BEZWZGLEDNA.search(linia):
                    wypisz(plik=p, linia=nr, regula='plik-ide', waga='ostrzega',
                           opis='ścieżka bezwzględna z jednej maszyny w pliku .vscode/ — użyj ${workspaceFolder}',
                           odcisk_tresci=linia.strip())
                    break
            continue
        if IDE_LUZNY.search(p):
            wypisz(plik=p, linia=None, regula='plik-ide', waga='ostrzega',
                   opis='plik tymczasowy edytora / systemu — dopisz do .gitignore', odcisk_tresci=p)
    grupami(grupy, 'plik-ide', 'ostrzega',
            lambda k: f'katalog IDE ({k}) w commicie — ustawienia osobiste, dopisz do .gitignore')


def pole_configu(root, sekcja, pole):
    """Wartość pola sekcji configu; None gdy nie ma configu albo sekcji, '' gdy nie ma pola."""
    try:
        with open(os.path.join(root, 'ralph', 'config.md'), encoding='utf-8') as f:
            t = re.sub(r'<!--.*?-->', '', f.read(), flags=re.S)
    except OSError:
        return None
    m = re.search(r'^## ' + re.escape(sekcja) + r'[ \t]*\n(.*?)(?=^## |\Z)', t, re.M | re.S)
    if not m:
        return None
    p = re.search(r'^- \*\*' + re.escape(pole) + r'\*\*:[ \t]*(.*?)[ \t]*$', m.group(1), re.M)
    return p.group(1) if p else ''


def artefakty(pliki, artefakty_dir, commit_artefaktow):
    if commit_artefaktow or not artefakty_dir:
        return
    grupy = {}
    for p in pliki:
        if p.startswith(artefakty_dir):
            grupy.setdefault(artefakty_dir.rstrip('/'), []).append(p)
    grupami(grupy, 'artefakty-wizualne', 'ostrzega',
            lambda k: f'artefakt wizualny w commicie przy „Commit artefakty: nie" — dopisz {k}/ do .gitignore '
                      f'albo ustaw „tak" w ralph/config.md (## Artefakty wizualne)')


def brak_gitignore(root, pliki):
    if os.path.isfile(os.path.join(root, '.gitignore')) or '.gitignore' in pliki:
        return
    kod = next((p for p in pliki if KOD.search(p)), None)
    if kod:
        wypisz(plik='.gitignore', linia=None, regula='brak-gitignore', waga='ostrzega', odcisk_pliku=False,
               opis=f'repozytorium bez .gitignore, a w commicie jest już kod ({kod}) — dodaj .gitignore '
                    f'(katalogi budowania, .env, artifacts/), zanim coś z tego trafi do historii',
               odcisk_tresci='brak-gitignore')


def main():
    punkt = os.environ.get('RALPH_PUNKT', 'commit')
    root = os.environ.get('RALPH_ROOT') or os.getcwd()
    ze_stdin = [l.strip() for l in sys.stdin.read().splitlines() if l.strip()]
    zakres = Zakres(root, punkt)
    pliki = sorted(p for p in set(ze_stdin) | zakres.dodatkowe()
                   if not p.startswith('ralph-kontrole/') and os.path.isfile(os.path.join(root, p)))
    if not pliki:
        return 0
    commit_art = pole_configu(root, 'Artefakty wizualne', 'Commit artefakty')
    art_dir = (pole_configu(root, 'Artefakty wizualne', 'Artifacts dir') or 'artifacts/').strip().rstrip('/') + '/' \
        if commit_art is not None else None
    commit_art = (commit_art or '').lower() in ('tak', 'yes', 'true')

    katalogi_budowania(root, pliki)
    rozmiary(root, pliki, art_dir, commit_art)
    lockfile_nieaktualny(root, pliki, zakres)
    wersje_plywajace(root, pliki)
    binaria(root, pliki)
    bundle(root, pliki)
    pliki_ide(root, pliki)
    artefakty(pliki, art_dir, commit_art)
    brak_gitignore(root, pliki)
    return 0


if __name__ == '__main__':
    sys.exit(main())
