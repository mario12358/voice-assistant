#!/usr/bin/env python3
"""Kontrola `red-team` — każda chroniona trasa zmieniona w fazie ma test, który ją atakuje.

Lista zmienionych plików na stdin (diff od poprzedniego tagu fazy), znaleziska JSON w liniach.
Dla tras z `ralph/BEZPIECZENSTWO.md` o roli innej niż `anonim`, których definicja albo ciało
zmieniło się w fazie, szuka w plikach testów znacznika:

    ralph: red-team <METODA> <ścieżka> — bez-sesji, obca-rola, obcy-wlasciciel, csrf

Wymagane przypadki wynikają z mapy, nie z pamięci: `bez-sesji` zawsze, `obca-rola` gdy mapa ma
więcej ról niż te, które trasa dopuszcza, `obcy-wlasciciel` gdy kolumna Własność ≠ —, `csrf`
dla zapisu przy sesji w ciasteczku. Brak → blokuje tag fazy, a opis znaleziska jest treścią
zadania. Znacznik nie dowodzi, że test cokolwiek pilnuje — to sprawdza mutacja (general.md)
i audytor (E4); ta kontrola pilnuje, że test w ogóle powstał.

Pierwsza faza po założeniu mapy tylko ostrzega: mapy nie było w punkcie odniesienia, więc
część tras ma role ze szkieletu, a projekt nie zdążył się z nią zapoznać.
"""
import hashlib
import json
import os
import re
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), '..'))
import _trasy as T  # noqa: E402

MAX_ZNALEZISK = 30
ZNACZNIK = re.compile(r'ralph:\s*red-team\s+([A-Za-z]+|\*)\s+(\S+)\s*(.*)$')
PRZYPADKI = ('bez-sesji', 'obca-rola', 'obcy-wlasciciel', 'csrf')
ALIASY = {'inna-organizacja': 'obcy-wlasciciel', 'obca-organizacja': 'obcy-wlasciciel',
          'obcy-właściciel': 'obcy-wlasciciel', 'bez-sesji': 'bez-sesji', 'anonim': 'bez-sesji',
          'obca-rola': 'obca-rola', 'obcy-wlasciciel': 'obcy-wlasciciel', 'csrf': 'csrf'}


def odcisk(t):
    return hashlib.sha256(t.encode('utf-8', 'replace')).hexdigest()[:16]


def wypisz(**z):
    print(json.dumps(z, ensure_ascii=False))


def zmienione_linie(root, od, p):
    """Numery linii (strona nowa) dotknięte w diffie od..HEAD; czyste usunięcie = linia, w której było."""
    d = T.git(root, 'diff', '-U0', '--no-color', od, 'HEAD', '--', p) or ''
    out = set()
    for m in re.finditer(r'^@@ -\d+(?:,\d+)? \+(\d+)(?:,(\d+))? @@', d, re.M):
        start, ile = int(m.group(1)), int(m.group(2) if m.group(2) is not None else 1)
        out.update(range(start, start + ile) if ile else (start, start + 1))
    return out


def znaczniki(root):
    """[(metoda, ścieżka, {przypadki}, plik)] ze wszystkich plików testów w repozytorium."""
    out = []
    for p in (T.git(root, 'ls-files') or '').splitlines():
        if not T.PLIK_TESTU.search(p) or T.POMIJANE_KATALOGI.search(p):
            continue
        t = T.wczytaj(root, p)
        if not t or 'red-team' not in t:
            continue
        for w in t.split('\n'):
            m = ZNACZNIK.search(w)
            if not m:
                continue
            slowa = re.findall(r'[\w-]+', m.group(3).lower())
            out.append((m.group(1).upper(), m.group(2).strip('`"\''), {ALIASY[s] for s in slowa if s in ALIASY}, p))
    return out


def pokrywa(znacznik, sciezka):
    """`/registrar/*` = każda trasa pod prefiksem — test, który iteruje po liście tras modułu
    (altaforta: „CSRF na 26 trasach zapisu"), nie musi mieć linii na każdą."""
    if znacznik.endswith('/*'):
        pref = [x for x in T.kanon(znacznik[:-2]).split('/') if x]
        seg = [x for x in T.kanon(sciezka).split('/') if x]
        return seg[:len(pref)] == pref
    return T.pasuje_sciezka(znacznik, sciezka)


def wymagane(trasa, wiersz, mapa):
    alt = T.alternatywy(wiersz['role']) or []
    potrzebne = ['bez-sesji']
    role_trasy = {r for a in alt for r in a}
    pozostale = {r for r in mapa['role'] if r != 'anonim'}
    if len(pozostale) >= 2 and not pozostale <= role_trasy:
        potrzebne.append('obca-rola')
    if not T.puste(wiersz.get('wlasnosc', '?')):
        potrzebne.append('obcy-wlasciciel')
    if trasa['metoda'] in T.METODY_ZAPISU and mapa['sesja'] not in ('nagłówek', 'naglowek', 'bearer', 'header'):
        potrzebne.append('csrf')
    return potrzebne


def main():
    root = os.environ.get('RALPH_ROOT') or os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
    if '--sprawdz' in sys.argv:
        return 0
    punkt = os.environ.get('RALPH_PUNKT', 'faza')
    pliki = [l.strip() for l in sys.stdin.read().splitlines() if l.strip()]
    pliki = [p for p in pliki if T.plik_kodu(p)]
    mapa = T.wczytaj_mape(root)
    if not pliki or mapa is None:
        return 0                                  # bez mapy nie wiadomo, co chronione — `intencja` to zgłasza
    od = T.punkt_odniesienia(root, 'faza' if punkt == 'commit' else punkt)
    pierwsza = od == T.PUSTE_DRZEWO or T.git(root, 'cat-file', '-e', f'{od}:{T.MAPA}') is None

    dotkniete = []
    for p in pliki:
        linie = zmienione_linie(root, od, p)
        if not linie:
            continue
        for tr in T.trasy_pliku(p, T.wczytaj(root, p) or '', root):
            if any(tr['od'] <= n <= tr['do'] for n in linie):
                dotkniete.append(tr)
    if not dotkniete:
        return 0
    zn = znaczniki(root)
    znal, nieustalone = [], 0
    for tr in sorted(dotkniete, key=lambda x: (x['plik'], x['linia'])):
        k = T.klucz(tr['metoda'], tr['sciezka'])
        wiersz = mapa['trasy'].get(k)
        if wiersz is None or T.publiczna(wiersz['role']):
            continue                              # brak wiersza zgłasza `intencja`; publiczna nie ma kogo wykluczyć
        if T.alternatywy(wiersz['role']) is None:
            nieustalone += 1
            continue
        potrzebne = wymagane(tr, wiersz, mapa)
        mam = set()
        for met, sc, przyp, _ in zn:
            if met in (tr['metoda'], 'ALL', '*') and pokrywa(sc, tr['sciezka']):
                mam |= przyp
        brak = [x for x in potrzebne if x not in mam]
        if not brak:
            continue
        znal.append(dict(plik=tr['plik'], linia=tr['linia'], regula='red-team-brak',
                         waga='ostrzega' if pierwsza else 'blokuje',
                         odcisk_tresci=odcisk(k + '|' + ','.join(brak)),
                         opis=(('(pierwsza faza z mapą — od następnej blokuje) ' if pierwsza else '')
                               + f'`{tr["metoda"]} {tr["sciezka"]}` zmieniona w fazie, brak red teamu: '
                               f'{", ".join(brak)} — test, który atakuje trasę i oczekuje odmowy, '
                               f'z linią `ralph: red-team {tr["metoda"]} {tr["sciezka"]} — {", ".join(potrzebne)}`')))
    if nieustalone:
        znal.append(dict(plik=T.MAPA, linia=None, regula='red-team-nieustalone', waga='ostrzega', odcisk_pliku=False,
                         odcisk_tresci=odcisk(f'nieustalone|{nieustalone}'),
                         opis=f'{nieustalone} tras zmienionych w fazie ma w mapie rolę „?" — red team ich nie '
                              f'obejmuje, dopóki rola nie zostanie ustalona'))
    for z in znal[:MAX_ZNALEZISK]:
        wypisz(**z)
    reszta = znal[MAX_ZNALEZISK:]
    if reszta:
        wypisz(plik=reszta[0]['plik'], linia=None, regula='red-team-brak', waga=reszta[0]['waga'],
               odcisk_pliku=False, odcisk_tresci='licznik',
               opis=f'+{len(reszta)} dalszych tras bez red teamu — pełna lista: '
                    f'python3 ralph-kontrole/ralph-kontrole.py --punkt {punkt}')
    return 0


if __name__ == '__main__':
    sys.exit(main())
