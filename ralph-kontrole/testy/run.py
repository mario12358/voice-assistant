#!/usr/bin/env python3
"""Kontrola `testy` — lista plików na stdin, znaleziska JSON w liniach na stdout.

Suite zielony dlatego, że część testów po cichu wyłączono, usunięto albo pozbawiono asercji,
jest gorszy od czerwonego — na czerwony ktoś patrzy. Ta kontrola łapie to przy commicie,
zanim trafi do historii. Pliki usunięte NIE przychodzą na stdin (runner daje tylko ACMR),
więc o usunięcia moduł pyta gita sam.
"""
import hashlib
import json
import os
import re
import subprocess
import sys

# Ta sama reguła co w oslabienia/run.py — jedna definicja „pliku testu" w całym frameworku
PLIK_TESTU = re.compile(r'(^|/)(tests?|__tests__|spec|e2e)/|(^|/)test_[^/]*$|[._-](test|spec)\.[^/]+$|_test\.[^/]+$')
KOD = re.compile(r'\.(py|js|jsx|ts|tsx|mjs|cjs|go|rs|java|kt|rb|php)$')
JS = re.compile(r'\.(js|jsx|ts|tsx|mjs|cjs)$')
# Test w TS może sprawdzać komponent w TSX — para po rdzeniu nazwy w obrębie rodziny języka
RODZINA = {'js': 'js', 'jsx': 'js', 'ts': 'js', 'tsx': 'js', 'mjs': 'js', 'cjs': 'js', 'java': 'jvm', 'kt': 'jvm'}

KATALOGI_KODU = ('src', 'app', 'lib', 'backend', 'frontend', 'server', 'client', 'api', 'pkg', 'internal',
                 'cmd', 'packages', 'apps', 'services', 'core', 'domain', 'web', 'mobile')
# Pliki bez logiki — zmiana bez testu to norma, nie sygnał
BEZ_LOGIKI = re.compile(r'(^|/)(\.[^/]*|[^/]*config[^/]*|settings\.py|conftest\.py|__init__\.py|setup\.py|manage\.py|'
                        r'types?\.(py|ts|tsx|js)|constants?\.(py|ts|js)|enums?\.(py|ts)|[^/]*\.d\.ts)$'
                        r'|(^|/)(migrations|alembic|scripts|tools|docs?|\.github)/', re.I)

# Helper asertujący za test (`self.sprawdz_blad(...)`, `self.check_response(...)`) liczy się jako
# asercja — pierwszy przebieg na testach benchmarku dał 10 fałszywych ostrzeżeń właśnie na nich
ASERCJA = re.compile(r'\bassert\w*\b|\bassert\w*!\(|\bexpect\s*\(|\bshould\b|\bt\.(Error|Fatal|Fail)\w*\(|'
                     r'\brequire\.\w+\(|pytest\.raises|\.toThrow|\.toMatchSnapshot|\.rejects\b|'
                     r'\bself\.\w*(sprawdz|check|verif|blad|oczekuj|ensure)\w*\(')
# Asercja, która niczego nie sprawdza — test jest zielony z definicji
PUSTA = re.compile(r'\bassert\s+(True|1)\s*(,|$|#)|expect\((true|1)\)\.(toBe|toEqual|toStrictEqual)\(\2\)|'
                   r'expect\(true\)\.toBeTruthy\(\)|assertTrue\(True\)|assert!\(true\)')
# `.only` wyłącza resztę suite'u — zielony wynik mówi o jednym teście. Celowo bez `fit:`
# (klucz obiektu `{ fit: 'cover' }` w fiksturze to nie fokus testu).
ONLY = re.compile(r'\.only\s*\(|\b(fit|fdescribe)\s*\(')
DEKORATOR_SKIP = re.compile(r'^\s*@((?:pytest\.)?mark\.skip(?:if)?|(?:unittest\.)?skip(?:If|Unless)?)\b(.*)$')
SKIP_JS = re.compile(r'\b(it|test|describe)\.skip\s*\(|\b(xit|xtest|xdescribe)\s*\(')
SKIP_GO_PUSTY = re.compile(r'\bt\.Skip\(\s*\)|\bt\.SkipNow\(\)')
SKIP_GO_Z_POWODEM = re.compile(r'\bt\.Skipf?\(\s*\S')
IGNORE_RS = re.compile(r'^\s*#\[ignore(\s*=\s*"[^"]+")?\s*\]')
DISABLED_JVM = re.compile(r'^\s*@(Disabled|Ignore)\b(\s*\(\s*"[^"]+")?')
# JS/Go nie mają miejsca na powód w API — powód to komentarz w tej samej linii albo linii wyżej
POWOD_W_KOMENTARZU = re.compile(r'(//|/\*|#)\s*(skip|pow[oó]d|reason)\s*:', re.I)


def odcisk(t):
    return hashlib.sha256(t.encode('utf-8', 'replace')).hexdigest()[:16]


def wypisz(**z):
    print(json.dumps(z, ensure_ascii=False))


def git(root, *args, timeout=15):
    try:
        r = subprocess.run(['git', '-C', root, *args], capture_output=True, text=True, timeout=timeout)
    except (OSError, subprocess.TimeoutExpired):
        return None
    return r.stdout if r.returncode == 0 else None


def linie(s):
    return [x for x in (s or '').splitlines() if x.strip()]


def wczytaj(root, p):
    """Wiersze pliku albo None (binarny / nie do odczytu)."""
    try:
        with open(os.path.join(root, p), 'rb') as f:
            dane = f.read()
    except OSError:
        return None
    if b'\0' in dane[:8192]:
        return None
    return dane.decode('utf-8', 'replace').splitlines()


# --- Nazwy: test ↔ moduł ------------------------------------------------------------

def rdzen(p):
    """(rdzeń nazwy bez sufiksu testowego, rodzina języka) — klucz parowania test ↔ kod."""
    nazwa = os.path.basename(p)
    stem, _, ext = nazwa.rpartition('.')
    if not stem:
        return None
    for wz in (r'^test_(.+)$', r'^(.+)[._-](test|spec)$', r'^(.+)(Test|Tests|Spec|IT)$'):
        m = re.match(wz, stem)
        if m:
            stem = m.group(1)
            break
    return stem, RODZINA.get(ext, ext)


def katalogi_kodu(root):
    """`- **Katalogi kodu**:` z podsekcji ### testy w configu albo domyślne."""
    try:
        with open(os.path.join(root, 'ralph', 'config.md'), encoding='utf-8') as f:
            tekst = f.read()
    except OSError:
        return KATALOGI_KODU
    m = re.search(r'^### testy[ \t]*\n(.*?)(?=^##|\Z)', tekst, re.M | re.S)
    if not m:
        return KATALOGI_KODU
    blok = re.sub(r'<!--.*?-->', '', m.group(1), flags=re.S)
    k = re.search(r'^- \*\*Katalogi kodu\*\*:[ \t]*(.+?)[ \t]*$', blok, re.M)
    if not k:
        return KATALOGI_KODU
    return tuple(x.strip().strip('`/') for x in k.group(1).split(',') if x.strip())


def plik_kodu(p, katalogi):
    """Plik z logiką w katalogu kodu (albo w korzeniu — płaski projekt)."""
    if not KOD.search(p) or PLIK_TESTU.search(p) or BEZ_LOGIKI.search(p):
        return False
    pierwszy = p.split('/', 1)[0] if '/' in p else ''
    return not pierwszy or pierwszy in katalogi


def reeksport(wiersze):
    """index.ts, który tylko re-eksportuje — bez logiki."""
    tresc = [l.strip() for l in wiersze if l.strip() and not l.strip().startswith(('//', '*', '/*'))]
    return bool(tresc) and all(l.startswith(('export ', 'export{', 'import ')) for l in tresc)


# --- Treść plików testów ------------------------------------------------------------

def dekorator_pelny(wiersze, i):
    """Tekst dekoratora z domknięciem nawiasów (reason= bywa w następnej linii)."""
    tekst, j = wiersze[i], i
    while tekst.count('(') > tekst.count(')') and j + 1 < len(wiersze) and j - i < 6:
        j += 1
        tekst += ' ' + wiersze[j].strip()
    return tekst


def skip_z_powodem(p, wiersze, nr, linia):
    """None = to nie skip; True = skip z powodem; False = skip bez powodu."""
    poprzednia = wiersze[nr - 2] if nr > 1 else ''
    if p.endswith('.py'):
        m = DEKORATOR_SKIP.match(linia)
        if not m:
            return None
        pelny = dekorator_pelny(wiersze, nr - 1)
        if re.search(r'\breason\s*=', pelny):
            return True
        # pytest.mark.skip("…") i unittest.skip("…") biorą powód pozycyjnie; skipif / skipIf —
        # pierwszy argument to warunek, powód tylko przez reason= (skipIf: drugi literał)
        if m.group(1).endswith('skipif'):
            return False        # pytest: reason tylko jako słowo kluczowe — a tego nie było
        literaly = re.findall(r'''["'][^"']{2,}["']''', m.group(2))
        return bool(literaly)   # skipIf(cond, "…") — drugi literał; skip("…") — pierwszy
    if JS.search(p):
        if not SKIP_JS.search(linia):
            return None
        return bool(POWOD_W_KOMENTARZU.search(linia) or POWOD_W_KOMENTARZU.search(poprzednia))
    if p.endswith('.go'):
        if SKIP_GO_PUSTY.search(linia):
            return bool(POWOD_W_KOMENTARZU.search(linia) or POWOD_W_KOMENTARZU.search(poprzednia))
        if SKIP_GO_Z_POWODEM.search(linia):
            return True
        return None
    if p.endswith('.rs'):
        m = IGNORE_RS.match(linia)
        return None if not m else bool(m.group(1))
    if p.endswith(('.java', '.kt')):
        m = DISABLED_JVM.match(linia)
        return None if not m else bool(m.group(2))
    return None


def cialo_klamrowe(wiersze, i, limit=400):
    """Linie od i do domknięcia pierwszej klamry (JS / Go) — bez parsera, po zliczaniu."""
    glebokosc, otwarte, j = 0, False, i
    while j < len(wiersze) and j - i < limit:
        glebokosc += wiersze[j].count('{') - wiersze[j].count('}')
        otwarte = otwarte or glebokosc > 0
        if otwarte and glebokosc <= 0:
            break
        j += 1
    return wiersze[i:j + 1]


def funkcje_testowe(p, wiersze):
    """[(nr, nazwa, linie ciała)] — testy, które coś wykonują (bez .skip / .todo)."""
    out = []
    for i, l in enumerate(wiersze):
        if p.endswith('.py'):
            m = re.match(r'^(\s*)(?:async\s+)?def\s+(test\w*)\s*\(', l)
            if not m:
                continue
            wciecie, j = len(m.group(1)), i + 1
            while j < len(wiersze) and (not wiersze[j].strip() or len(wiersze[j]) - len(wiersze[j].lstrip()) > wciecie):
                j += 1
            out.append((i + 1, m.group(2), wiersze[i:j]))
        elif JS.search(p):
            m = re.match(r'''^\s*(it|test)((?:\.\w+)*)\s*[(`]\s*(?:['"`]([^'"`]*))?''', l)
            if not m or '.skip' in m.group(2) or '.todo' in m.group(2):
                continue
            if '{' not in l and '=>' not in l and 'function' not in l:
                continue        # it('x', sprawdzFoo) — treść poza zasięgiem, nie zgadujemy
            out.append((i + 1, m.group(3) or 'it', cialo_klamrowe(wiersze, i)))
        elif p.endswith('_test.go'):
            m = re.match(r'^func\s+(Test\w+)\s*\(', l)
            if m:
                out.append((i + 1, m.group(1), cialo_klamrowe(wiersze, i)))
    return out


def sprawdz_plik_testu(p, wiersze, trafienia):
    for nr, linia in enumerate(wiersze, 1):
        if len(linia) > 4000:
            continue
        if JS.search(p) and ONLY.search(linia):
            trafienia.setdefault('only', []).append((nr, linia.strip(), None))
        powod = skip_z_powodem(p, wiersze, nr, linia)
        if powod is True:
            trafienia.setdefault('skip-z-powodem', []).append((nr, linia.strip(), None))
        elif powod is False:
            trafienia.setdefault('skip-bez-powodu', []).append((nr, linia.strip(), None))
    for nr, nazwa, cialo in funkcje_testowe(p, wiersze):
        prawdziwe = [l for l in cialo if ASERCJA.search(l) and not PUSTA.search(l)]
        if prawdziwe:
            continue
        if any(PUSTA.search(l) for l in cialo):
            trafienia.setdefault('test-bez-asercji', []).append((nr, nazwa, 'asercja pusta (assert True / expect(true).toBe(true))'))
        else:
            trafienia.setdefault('test-bez-asercji', []).append((nr, nazwa, 'bez żadnej asercji'))


OPISY = {
    'only': ('blokuje', '`.only` / `fit` wyłącza resztę suite\'u — zielony wynik mówi o jednym teście; usuń przed commitem'),
    'skip-bez-powodu': ('blokuje', 'test wyłączony bez powodu — dodaj powód (reason="…" / komentarz `// skip: …` linię wyżej) '
                                   'albo przywróć test'),
    'skip-z-powodem': ('ostrzega', 'test wyłączony z powodem — pamiętaj, że suite jest zielony bez niego'),
    'test-bez-asercji': ('ostrzega', 'test {szczegol} — przechodzi niezależnie od implementacji; sprawdź, '
                                     'co ma dowodzić, i dopisz asercję'),
}


def wypisz_trafienia(p, trafienia):
    """Ten sam wzorzec w 15 liniach to jedno znalezisko z licznikiem."""
    for regula, lista in trafienia.items():
        nr, klucz, szczegol = lista[0]
        waga, opis = OPISY[regula]
        dalej = f' (+{len(lista) - 1} dalej w pliku)' if len(lista) > 1 else ''
        if regula == 'test-bez-asercji':
            opis = opis.format(szczegol=f'`{klucz[:60]}` {szczegol}')
        else:
            opis = f'{opis}: `{klucz[:80]}`'
        wypisz(plik=p, linia=nr, regula=regula, waga=waga, opis=opis + dalej, odcisk_tresci=odcisk(klucz))


# --- Stan w gicie: baza porównania, statusy, usunięcia ------------------------------------

def baza_porownania(root, punkt):
    """Rewizja „przed zmianą": przy commit HEAD; w CI merge-base z RALPH_OD / origin/main.
    None = nie ma z czym porównać (reguły zależne od poprzedniej wersji są pomijane)."""
    if punkt != 'ci':
        return 'HEAD' if git(root, 'rev-parse', '--verify', '-q', 'HEAD^{commit}') else None
    kandydaci = [os.environ.get('RALPH_OD')] + ['origin/main', 'origin/master']
    for k in kandydaci:
        if k and git(root, 'rev-parse', '--verify', '-q', k + '^{commit}'):
            mb = git(root, 'merge-base', k, 'HEAD')
            return mb.strip() if mb else None
    return None


def usuniete(root, punkt, baza):
    if punkt == 'ci':
        return set(linie(git(root, 'diff', '--name-only', '--diff-filter=D', baza, 'HEAD')))
    # `git rm` → w indeksie; `rm` + `git add -A` w komendzie → jeszcze tylko w drzewie roboczym
    return (set(linie(git(root, 'diff', '--cached', '--name-only', '--diff-filter=D')))
            | set(linie(git(root, 'diff', '--name-only', '--diff-filter=D', 'HEAD'))))


# Usunięta linia, która nie jest treścią testu: import rozszerzony o kolejną nazwę
# (`import { render, screen }` → `… within }` na Specky dał fałszywe „przypięcie")
IMPORT_LUB_PUSTA = re.compile(r'^\s*(import\b|from\s+\S+\s+import\b|const\s+\{[^}]*\}\s*=\s*require\(|'
                              r'[})\]];?\s*$|$|#|//)')


def usuniete_linie_testu(root, punkt, baza, p):
    """Linie usunięte z pliku testu, które niosą treść testu (bez importów i pustych)."""
    args = ['diff', '-U0'] + (['HEAD'] if punkt != 'ci' else [baza, 'HEAD']) + ['--', p]
    wynik = git(root, *args) or ''
    return [l[1:] for l in wynik.splitlines()
            if l.startswith('-') and not l.startswith('---') and not IMPORT_LUB_PUSTA.match(l[1:])]




def licz_asercje(wiersze):
    return sum(len(ASERCJA.findall(l)) for l in wiersze if len(l) <= 4000)


# --- Główna pętla ------------------------------------------------------------------------

def main():
    punkt = os.environ.get('RALPH_PUNKT', 'commit')
    root = os.environ.get('RALPH_ROOT') or os.getcwd()
    pliki = [l.strip() for l in sys.stdin.read().splitlines() if l.strip()]
    katalogi = katalogi_kodu(root)
    baza = baza_porownania(root, punkt)
    w_bazie = set(linie(git(root, 'ls-tree', '-r', '--name-only', baza))) if baza else set()
    status = {p: ('M' if p in w_bazie else 'A') for p in pliki}

    testy, kod = {}, {}     # rdzeń → ścieżka
    for p in pliki:
        wiersze = wczytaj(root, p)
        if wiersze is None:
            continue
        if PLIK_TESTU.search(p) or p.endswith('.rs'):
            trafienia = {}
            sprawdz_plik_testu(p, wiersze, trafienia)
            wypisz_trafienia(p, trafienia)
        if not PLIK_TESTU.search(p):
            if plik_kodu(p, katalogi) and not (os.path.basename(p).startswith('index.') and reeksport(wiersze)):
                kod[rdzen(p)] = p
            continue
        testy[rdzen(p)] = p
        if status[p] == 'M' and baza:
            przed = git(root, 'show', f'{baza}:{p}')
            if przed is not None:
                a, b = licz_asercje(przed.splitlines()), licz_asercje(wiersze)
                if b < a:
                    wypisz(plik=p, linia=None, regula='mniej-asercji', waga='ostrzega',
                           opis=f'liczba asercji spadła {a} → {b} — jeśli test został osłabiony, '
                                f'żeby przeszedł, to implementacja ma błąd, nie test',
                           odcisk_tresci=f'{a}>{b}')

    # kod-bez-testu: jedno znalezisko na commit, odcisk z listy plików (nie z miejsca)
    if kod and not testy:
        lista = sorted(kod.values())
        wypisz(plik=lista[0], linia=None, regula='kod-bez-testu', waga='ostrzega', odcisk_pliku=False,
               opis=f'{len(lista)} plik(ów) kodu bez żadnego pliku testu w commicie: '
                    f'{", ".join(lista[:4])}{" …" if len(lista) > 4 else ""} — '
                    f'refaktor bez zmiany zachowania jest w porządku; nowa logika nie',
               odcisk_tresci=odcisk('\n'.join(lista)))

    # test-przypiety: para kod ↔ test, oba modyfikowane, w teście coś usunięto
    for klucz, t in testy.items():
        k = kod.get(klucz)
        if not k or status[t] != 'M' or status[k] != 'M' or not baza:
            continue
        if not usuniete_linie_testu(root, punkt, baza, t):
            continue
        wypisz(plik=t, linia=None, regula='test-przypiety', waga='ostrzega',
               opis=f'test zmieniony razem z kodem, który sprawdza ({k}) — jeśli zmiana testu wynika '
                    f'z implementacji, nie z reguły, test pilnuje implementacji',
               odcisk_tresci=odcisk(f'{t}|{k}'))

    # test-usuniety: usunięte pliki nie przychodzą na stdin — pytamy gita
    if baza:
        skasowane = usuniete(root, punkt, baza)
        istniejace = {}
        for p in w_bazie - skasowane:
            if KOD.search(p) and not PLIK_TESTU.search(p):
                istniejace.setdefault(rdzen(p), p)
        for t in sorted(skasowane):
            if not PLIK_TESTU.search(t) or not KOD.search(t):
                continue
            k = istniejace.get(rdzen(t))
            if not k:
                continue        # testowany moduł też znika (albo nie da się go wskazać) — porządek
            wypisz(plik=t, linia=None, regula='test-usuniety', waga='blokuje',
                   opis=f'plik testu usunięty, a testowany moduł {k} zostaje — przywróć test '
                        f'albo przenieś jego przypadki; usunięcie testu nie naprawia kodu',
                   odcisk_tresci=odcisk(f'{t}|{k}'))
    return 0


if __name__ == '__main__':
    sys.exit(main())
