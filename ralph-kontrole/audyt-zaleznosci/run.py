#!/usr/bin/env python3
"""Kontrola `audyt-zaleznosci` — wersje z lockfile'i vs baza podatności OSV.

Ignoruje listę plików ze stdin: podatność pojawia się w bazie bez zmiany w projekcie,
więc zawsze czyta wszystkie śledzone lockfile'e. Znaleziska JSON w liniach na stdout.
"""
import hashlib
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
except ImportError:
    tomllib = None

POWAZNE = ('CRITICAL', 'HIGH')


def wypisz(**z):
    print(json.dumps(z, ensure_ascii=False))


def norm_py(n):
    return re.sub(r'[-_.]+', '-', n).lower()


# --- Lockfile'e → [(ekosystem OSV, nazwa, wersja, dev)] -----------------------------

def npm_lock(t):
    d = json.loads(t)
    out = []
    for sciezka, p in (d.get('packages') or {}).items():
        if not sciezka or not isinstance(p, dict) or p.get('link'):
            continue
        nazwa = p.get('name') or sciezka.split('node_modules/')[-1]
        if p.get('version'):
            out.append(('npm', nazwa, p['version'], bool(p.get('dev'))))
    if not out:                                   # lockfileVersion 1
        def chodz(deps):
            for n, p in (deps or {}).items():
                if isinstance(p, dict) and p.get('version'):
                    out.append(('npm', n, p['version'], bool(p.get('dev'))))
                    chodz(p.get('dependencies'))
        chodz(d.get('dependencies'))
    return out


def pnpm_lock(t):
    out = []
    for m in re.finditer(r"^\s{2}'?/?((?:@[^/@\s]+/)?[^@/\s:']+)@(\d[^(:'\s]*)", t, re.M):
        out.append(('npm', m.group(1), m.group(2), False))
    return out


def yarn_lock(t):
    out = []
    for blok in re.split(r'\n\s*\n', t):
        m = re.match(r'^"?((?:@[^@/\s]+/)?[^@\s"]+)@', blok.strip())
        w = re.search(r'^\s+version:?\s+"?([^"\s]+)"?', blok, re.M)
        if m and w:
            out.append(('npm', m.group(1), w.group(1), False))
    return out


def deno_lock(t):
    d = json.loads(t)
    npm = d.get('npm') or (d.get('packages') or {}).get('npm') or {}
    out = []
    for klucz in npm:
        m = re.match(r'^((?:@[^@/]+/)?[^@]+)@([^_]+)', klucz)
        if m:
            out.append(('npm', m.group(1), m.group(2), False))
    return out


def toml_lock(t):
    if tomllib is None:
        return []
    d = tomllib.loads(t)
    out = []
    for p in d.get('package') or []:
        if p.get('name') and p.get('version'):
            zrodlo = p.get('source') or {}
            if isinstance(zrodlo, dict) and (zrodlo.get('editable') or zrodlo.get('virtual')
                                              or zrodlo.get('type') in ('directory', 'git', 'file')):
                continue                           # pakiet lokalny / z gita — nie ma go w OSV
            out.append(('PyPI', norm_py(p['name']), p['version'], p.get('category') == 'dev'))
    return out


def pipfile_lock(t):
    d = json.loads(t)
    out = []
    for grupa, dev in (('default', False), ('develop', True)):
        for n, p in (d.get(grupa) or {}).items():
            w = (p or {}).get('version', '')
            if w.startswith('=='):
                out.append(('PyPI', norm_py(n), w[2:], dev))
    return out


def requirements(t):
    out = []
    for l in t.splitlines():
        m = re.match(r'^\s*([A-Za-z0-9][A-Za-z0-9._-]*)(\[[^\]]*\])?\s*==\s*([^\s;#,]+)', l)
        if m:
            out.append(('PyPI', norm_py(m.group(1)), m.group(3), False))
    return out


CZYTNIKI = [
    (re.compile(r'(^|/)(package-lock|npm-shrinkwrap)\.json$'), npm_lock),
    (re.compile(r'(^|/)pnpm-lock\.yaml$'), pnpm_lock),
    (re.compile(r'(^|/)yarn\.lock$'), yarn_lock),
    (re.compile(r'(^|/)deno\.lock$'), deno_lock),
    (re.compile(r'(^|/)(poetry|uv)\.lock$'), toml_lock),
    (re.compile(r'(^|/)Pipfile\.lock$'), pipfile_lock),
    (re.compile(r'(^|/)requirements[^/]*\.txt$'), requirements),
]


def lockfile_y(root):
    try:
        r = subprocess.run(['git', '-C', root, 'ls-files'], capture_output=True, text=True, timeout=30)
        pliki = r.stdout.splitlines() if r.returncode == 0 else []
    except (OSError, subprocess.TimeoutExpired):
        pliki = []
    for p in pliki:
        for wz, czytnik in CZYTNIKI:
            if wz.search(p):
                yield p, czytnik
                break


# --- OSV -------------------------------------------------------------------------------

def powaga(v):
    s = ((v.get('database_specific') or {}).get('severity') or '').upper()
    if s == 'MODERATE':
        s = 'MEDIUM'
    if s:
        return s
    for a in v.get('affected') or []:
        s = str(((a.get('ecosystem_specific') or {}).get('severity')) or '').upper()
        if s:
            return 'MEDIUM' if s == 'MODERATE' else s
    return ''


def poprawki(v, eko, nazwa):
    out = []
    for a in v.get('affected') or []:
        pk = a.get('package') or {}
        n = norm_py(pk.get('name', '')) if eko == 'PyPI' else pk.get('name')
        if pk.get('ecosystem') != eko or n != nazwa:
            continue
        for r in a.get('ranges') or []:
            for e in r.get('events') or []:
                if e.get('fixed'):
                    out.append(e['fixed'])
    return out


def klucz_wersji(w):
    """Porównywalna krotka z wersji (1.10.2 > 1.9.0; sufiksy jak rc/post spychane w dół)."""
    czesci = []
    for c in re.split(r'[.+-]', str(w)):
        m = re.match(r'^(\d+)(.*)$', c)
        czesci.append((int(m.group(1)), m.group(2)) if m else (-1, c))
    return czesci


def wyzsze(poprawki_, wersja):
    """Tylko poprawki nowsze od zainstalowanej — OSV podaje je per gałąź (1.88.6, 1.89.7…),
    a gałęzi starszej od zainstalowanej wersji nie da się użyć."""
    kw = klucz_wersji(wersja)
    return sorted({f for f in poprawki_ if klucz_wersji(f) > kw}, key=klucz_wersji)


def grupuj(ids, szczegoly):
    """Ten sam błąd bywa w OSV pod dwoma id (GHSA i PYSEC z tym samym CVE). Łączy po
    aliasach; reprezentantem grupy jest wpis z oceną powagi."""
    grupy = []
    for vid in ids:
        v = szczegoly.get(vid) or {'id': vid}
        nazwy = {vid, *(v.get('aliases') or [])}
        for g in grupy:
            if g['nazwy'] & nazwy:
                g['nazwy'] |= nazwy
                g['wpisy'].append(v)
                break
        else:
            grupy.append({'nazwy': nazwy, 'wpisy': [v]})
    for g in grupy:
        g['wpisy'].sort(key=lambda v: (not powaga(v), not v['id'].startswith('GHSA'), v['id']))
    return grupy


RANGA = {'CRITICAL': 4, 'HIGH': 3, 'MEDIUM': 2, 'LOW': 1, '': 0}


def main():
    root = os.environ.get('RALPH_ROOT') or os.getcwd()
    sys.stdin.read()
    pakiety = {}                          # (eko, nazwa, wersja) → (plik, dev)
    for plik, czytnik in lockfile_y(root):
        try:
            with open(os.path.join(root, plik), encoding='utf-8') as f:
                lista = czytnik(f.read())
        except (OSError, ValueError, Exception):
            wypisz(plik=plik, linia=None, regula='nieczytelny-lockfile', waga='ostrzega',
                   opis='nie udało się odczytać lockfile — jego zależności nie zostały sprawdzone',
                   odcisk_tresci=plik)
            continue
        for eko, nazwa, wersja, dev in lista:
            k = (eko, nazwa, wersja)
            if k not in pakiety or (pakiety[k][1] and not dev):
                pakiety[k] = (plik, dev)      # produkcyjna wygrywa z deweloperską
    if not pakiety:
        return 0

    pamiec = Pamiec('osv')
    klucze = sorted(pakiety)
    podatnosci = {}                       # (eko, nazwa, wersja) → [id]
    do_zapytania = []
    for k in klucze:
        z = pamiec.daj('q:' + '|'.join(k))
        if z is None:
            do_zapytania.append(k)
        else:
            podatnosci[k] = z
    try:
        for i in range(0, len(do_zapytania), 500):
            paczka = do_zapytania[i:i + 500]
            _, d = zapytaj(f'{adres("osv")}/v1/querybatch', {'queries': [
                {'package': {'ecosystem': e, 'name': n}, 'version': w} for e, n, w in paczka]}, limit=30)
            for k, wynik in zip(paczka, (d or {}).get('results') or []):
                ids = sorted({v['id'] for v in (wynik or {}).get('vulns') or [] if v.get('id')})
                podatnosci[k] = ids
                pamiec.wstaw('q:' + '|'.join(k), ids, 12 * 3600)
    except BrakSieci as e:
        pamiec.zapisz()
        wypisz(plik=next(iter(pakiety.values()))[0], linia=None, regula='nie-sprawdzono', waga='ostrzega',
               opis=f'baza OSV nieosiągalna — {len(do_zapytania)} pakietów niesprawdzonych ({str(e)[:80]})',
               odcisk_tresci='nie-sprawdzono')
        return 0

    szczegoly = {}

    def pobierz(vid):
        z = pamiec.daj('v:' + vid)
        if z is not None:
            return vid, z, False
        try:
            _, z = zapytaj(f'{adres("osv")}/v1/vulns/{vid}')
        except BrakSieci:
            z = None
        return vid, z, True

    with ThreadPoolExecutor(max_workers=8) as ex:
        for vid, z, nowe in ex.map(pobierz, sorted({i for ids in podatnosci.values() for i in ids})):
            if z and nowe:
                pamiec.wstaw('v:' + vid, z, 12 * 3600)
            szczegoly[vid] = z or {'id': vid}
    pamiec.zapisz()

    # Jedno znalezisko na pakiet: decyzja i tak jest jedna (podbić wersję albo zaakceptować
    # ryzyko), a pyjwt z 12 podatnościami to nie 12 problemów. Odcisk obejmuje listę
    # podatności — nowa podatność w tym samym pakiecie zablokuje od nowa mimo wyjątku.
    for k in klucze:
        eko, nazwa, wersja = k
        plik, dev = pakiety[k]
        grupy = grupuj(podatnosci.get(k, []), szczegoly)
        if not grupy:
            continue
        opisy, fixy, max_sev, blokuje = [], set(), '', False
        for g in sorted(grupy, key=lambda g: -RANGA.get(powaga(g['wpisy'][0]), 0)):
            v = g['wpisy'][0]
            sev = powaga(v)
            fix = wyzsze([f for w in g['wpisy'] for f in poprawki(w, eko, nazwa)], wersja)
            if RANGA.get(sev, 0) > RANGA.get(max_sev, 0):
                max_sev = sev
            if fix:
                fixy.add(fix[0])
            if sev in POWAZNE and fix and not dev:
                blokuje = True
            cve = next((a for a in sorted(g['nazwy']) if a.startswith('CVE-')), v['id'])
            opisy.append(f'{cve} ({sev.lower() or "bez oceny"}{"" if fix else ", bez poprawki"})')
        cel = max(fixy, key=klucz_wersji) if fixy else ''
        opis = (f'{nazwa} {wersja}: {len(grupy)} podatn. (najwyższa: {max_sev.lower() or "bez oceny"})'
                + (f' — podbij do ≥ {cel}' if cel else ' — brak wersji z poprawką')
                + (' — zależność deweloperska' if dev else '')
                + ': ' + ', '.join(opisy[:6]) + (' …' if len(opisy) > 6 else ''))
        odc = hashlib.sha1('|'.join(sorted(n for g in grupy for n in g['nazwy'])).encode()).hexdigest()[:12]
        wypisz(plik=plik, linia=None, regula='podatnosc', waga='blokuje' if blokuje else 'ostrzega',
               opis=opis, odcisk_tresci=f'{eko}:{nazwa}:{wersja}:{odc}')
    return 0


if __name__ == '__main__':
    sys.exit(main())
