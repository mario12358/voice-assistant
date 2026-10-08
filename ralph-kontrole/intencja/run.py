#!/usr/bin/env python3
"""Kontrola `intencja` — kod robi coś, czego mapa powierzchni ataku nie przewiduje.

Lista zmienionych plików na stdin, znaleziska JSON w liniach. Porównuje trasy, strażników,
hosty wychodzące i sekrety w zmienionych plikach z `ralph/BEZPIECZENSTWO.md`. „Nowe" znaczy:
nieobecne w tym pliku w punkcie odniesienia (commit → HEAD, faza → ostatni tag fazy) — więc
kontrola nie zależy od tego, co już jest w indeksie gita.

  --szkielet [--wypisz]  szkielet mapy z całego repozytorium (ralph-start.sh, gdy mapy brak);
                         --wypisz = na stdout zamiast do pliku
  --sprawdz              walidator przy starcie
"""
import hashlib
import json
import os
import re
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), '..'))
import _trasy as T  # noqa: E402

MAX_ZNALEZISK = 30
HOST = re.compile(r'https?://([A-Za-z0-9-]+(?:\.[A-Za-z0-9-]+)+)')
HOST_POMIJANY = re.compile(r'^(localhost|127\.\d+\.\d+\.\d+|0\.0\.0\.0|\[?::1\]?)$|(^|\.)(example\.(com|org|net)|'
                           r'test|local|localhost|invalid|internal|w3\.org|schema\.org|json-schema\.org|purl\.org|'
                           r'ogp\.me|xmlns\.com|openapis\.org)$', re.I)
ENV = re.compile(r'''os\.environ(?:\.get)?\s*[\[(]\s*['"](\w+)|getenv\(\s*['"](\w+)|process\.env\.(\w+)|'''
                 r'''process\.env\[\s*['"](\w+)|Deno\.env\.get\(\s*['"](\w+)|import\.meta\.env\.(\w+)|'''
                 r'''os\.Getenv\(\s*"(\w+)|System\.getenv\(\s*"(\w+)|ENV\[\s*['"](\w+)''')
SEKRETNA_NAZWA = re.compile(r'KEY|SECRET|TOKEN|PASSWORD|PASSWD|PRIVATE|CREDENTIAL|DSN|DATABASE_URL|_PAT$', re.I)
BAZA = re.compile(r'\bGRANT\b|SECURITY\s+DEFINER|\bBYPASSRLS\b|DISABLE\s+ROW\s+LEVEL\s+SECURITY|'
                  r'NO\s+FORCE\s+ROW\s+LEVEL|DROP\s+POLICY|ALTER\s+ROLE\b|\bSUPERUSER\b', re.I)
BLOB = re.compile(r'''['"`]([A-Za-z0-9+/_-]{200,}={0,2}|[0-9a-fA-F]{200,})['"`]''')
KOMENTARZ = re.compile(r'^\s*(#|//|\*|/\*|<!--|--)')


def odcisk(t):
    return hashlib.sha256(t.encode('utf-8', 'replace')).hexdigest()[:16]


def wypisz(**z):
    print(json.dumps(z, ensure_ascii=False))


def katalog_glowny():
    return os.environ.get('RALPH_ROOT') or os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


# --- Szkielet ---------------------------------------------------------------------------------------

def hosty_i_env(tekst):
    hosty, env = set(), set()
    for w in tekst.split('\n'):
        if KOMENTARZ.match(w):
            continue
        for m in HOST.finditer(w):
            h = m.group(1).lower()
            if not HOST_POMIJANY.search(h):
                hosty.add(h)
        for m in ENV.finditer(w):
            n = next(g for g in m.groups() if g)
            if SEKRETNA_NAZWA.search(n):
                env.add(n)
    return hosty, env


def szkielet(root, wypisz_na_stdout=False):
    sciezka = os.path.join(root, T.MAPA)
    if os.path.exists(sciezka) and not wypisz_na_stdout:
        return 0
    pliki = (T.git(root, 'ls-files') or '').splitlines() \
        + (T.git(root, 'ls-files', '--others', '--exclude-standard') or '').splitlines()
    pliki = sorted({p for p in pliki if T.plik_kodu(p)})
    trasy, hosty, env, ciasteczko = [], set(), set(), False
    for p in pliki:
        t = T.wczytaj(root, p)
        if not t:
            continue
        trasy += T.trasy_pliku(p, t, root)
        h, e = hosty_i_env(t)
        hosty |= h
        env |= e
        if re.search(r'set_cookie|\.cookie\(|setCookie|httponly|httpOnly|SameSite|cookie_auth|CookieTransport', t):
            ciasteczko = True
    role, wiersze = [], []
    for tr in sorted(trasy, key=lambda x: (x['plik'], x['linia'])):
        kand = T.kandydaci(tr)
        for k in kand:
            if k not in role:
                role.append(k)
        rola = ' + '.join(r for r, _ in kand) if kand else '?'
        wl = '?' if re.search(r'\{|:\w|<\w|\[\w|\$\{', tr['sciezka']) else '—'
        wiersze.append(f"| {tr['metoda']} {tr['sciezka']} | {rola} | {wl} | {tr['plik']} |")
    with open(os.path.join(os.path.dirname(os.path.abspath(__file__)), 'MAPA_SZABLON.md'), encoding='utf-8') as f:
        szablon = f.read()
    szablon = szablon.replace('- **Sesja**: ?', f"- **Sesja**: {'ciasteczko' if ciasteczko else '?'}", 1)
    wiersze_rol = [f'| {r} | ? | `{tok}` |' for r, tok in role]
    wiersze_wyjsc = [f'| host | {h} | ? |' for h in sorted(hosty)] + [f'| env | {e} | ? |' for e in sorted(env)]

    def dopisz(tekst, naglowek, nowe):
        if not nowe:
            return tekst
        i = tekst.index(naglowek)
        j = tekst.index('\n', tekst.index('\n', i) + 1) + 1          # za wierszem |---|
        while tekst.startswith('|', j):                               # za istniejącymi wierszami (anonim)
            j = tekst.index('\n', j) + 1
        return tekst[:j] + '\n'.join(nowe) + '\n' + tekst[j:]
    tekst = dopisz(szablon, '| Rola | Kto | Strażnik w kodzie |', wiersze_rol)
    tekst = dopisz(tekst, '| Trasa | Role | Własność | Plik |', wiersze)
    tekst = dopisz(tekst, '| Rodzaj | Wartość | Po co |', wiersze_wyjsc)
    nieust = sum(1 for w in wiersze if w.split('|')[2].strip() == '?')
    podsum = (f'{len(trasy)} tras ({nieust} bez rozpoznanego strażnika — rola „?"), {len(role)} ról-kandydatów, '
              f'{len(hosty)} hostów, {len(env)} sekretów ze środowiska')
    if wypisz_na_stdout:
        sys.stdout.write(tekst)
        print(podsum, file=sys.stderr)
        return 0
    if not trasy:
        return 0                                  # projekt bez tras HTTP — mapa nie jest potrzebna
    os.makedirs(os.path.dirname(sciezka), exist_ok=True)
    with open(sciezka, 'w', encoding='utf-8') as f:
        f.write(tekst)
    print(podsum)
    return 0


# --- Sprawdzenie ------------------------------------------------------------------------------------

def spelnia(trasa, alt, mapa):
    """Czy okno trasy zawiera strażnika każdej roli z tej alternatywy."""
    for rola in alt:
        if rola == 'anonim' or rola in mapa['globalne']:
            continue
        tokeny = mapa['role'].get(rola)
        if not tokeny:
            return None                           # rola bez strażnika — nie da się ocenić
        if not any(T.zawiera_straznika(trasa['okno'], tok) for tok in tokeny):
            return False
    return True


def nowe_linie(stary, nowy):
    znane = set(l.strip() for l in stary.split('\n'))
    return [(i + 1, l) for i, l in enumerate(nowy.split('\n')) if l.strip() and l.strip() not in znane]


def main():
    root = katalog_glowny()
    if '--szkielet' in sys.argv:
        return szkielet(root, '--wypisz' in sys.argv)
    if '--sprawdz' in sys.argv:
        if T.wczytaj(root, T.MAPA) is None and any(
                T.trasy_pliku(p, T.wczytaj(root, p) or '')
                for p in (T.git(root, 'ls-files') or '').splitlines() if T.plik_kodu(p)):
            print(f'uwaga: brak {T.MAPA} — ralph-start.sh wygeneruje szkielet przy starcie '
                  f'(python3 ralph-kontrole/intencja/run.py --szkielet)')
        return 0
    punkt = os.environ.get('RALPH_PUNKT', 'commit')
    pliki = [l.strip() for l in sys.stdin.read().splitlines() if l.strip()]
    pliki = [p for p in pliki if T.plik_kodu(p) or (T.MIGRACJA.search(p) and not T.POMIJANE_KATALOGI.search(p))]
    if not pliki:
        return 0
    od = T.punkt_odniesienia(root, punkt)
    mapa = T.wczytaj_mape(root)
    znal = []

    teraz, przedtem = {}, {}                      # klucz → trasa
    nowe_tresci, stare_tresci = {}, {}
    for p in pliki:
        nowy = T.wczytaj(root, p) or ''
        stary = T.tresc_z_rewizji(root, od, p)
        nowe_tresci[p], stare_tresci[p] = nowy, stary
        if T.plik_kodu(p):
            for tr in T.trasy_pliku(p, nowy, root):
                teraz.setdefault(T.klucz(tr['metoda'], tr['sciezka']), tr)
            for tr in T.trasy_pliku(p, stary, root):
                przedtem.setdefault(T.klucz(tr['metoda'], tr['sciezka']), tr)

    if mapa is None:
        if teraz:
            wypisz(plik=T.MAPA, linia=None, regula='brak-mapy', waga='ostrzega', odcisk_pliku=False,
                   odcisk_tresci='brak-mapy',
                   opis=f'commit zmienia {len(teraz)} tras HTTP, a mapy powierzchni ataku nie ma — trasy nie są '
                        f'sprawdzane; szkielet: python3 ralph-kontrole/intencja/run.py --szkielet')
        return 0

    nieznane_role, role_bez_straznika, nieustalone = set(), set(), {}
    for k, tr in sorted(teraz.items(), key=lambda x: (x[1]['plik'], x[1]['linia'])):
        wiersz = mapa['trasy'].get(k)
        nowa = k not in przedtem
        if wiersz is None:
            if nowa:
                znal.append(dict(plik=tr['plik'], linia=tr['linia'], regula='trasa-poza-mapa', waga='blokuje',
                                 odcisk_tresci=odcisk(k),
                                 opis=f'nowa trasa `{tr["metoda"]} {tr["sciezka"]}` bez wiersza w {T.MAPA} → '
                                      f'## Trasy — dopisz w tym samym commicie, kto ma do niej dostęp (Role) '
                                      f'i czy dotyczy cudzego zasobu (Własność)'))
            else:
                znal.append(dict(plik=tr['plik'], linia=tr['linia'], regula='trasa-poza-mapa', waga='ostrzega',
                                 odcisk_tresci=odcisk(k),
                                 opis=f'trasa `{tr["metoda"]} {tr["sciezka"]}` (sprzed tej zmiany) nie ma wiersza '
                                      f'w {T.MAPA} — dopisz go'))
            continue
        alt = T.alternatywy(wiersz['role'])
        if alt is None:
            nieustalone.setdefault(tr['plik'], []).append(f'{tr["metoda"]} {tr["sciezka"]}')
            continue
        for a in alt:
            for r in a:
                if r != 'anonim' and r not in mapa['role']:
                    nieznane_role.add(r)
        if T.publiczna(wiersz['role']):
            if nowa:
                znal.append(dict(plik=tr['plik'], linia=tr['linia'], regula='nowa-publiczna', waga='ostrzega',
                                 odcisk_tresci=odcisk(k),
                                 opis=f'nowa trasa publiczna `{tr["metoda"]} {tr["sciezka"]}` (rola anonim) — '
                                      f'odnotuj w raporcie, dlaczego bez sesji (webhook z podpisem, health…)'))
            continue
        wyniki = [spelnia(tr, a, mapa) for a in alt]
        if any(w is True for w in wyniki):
            continue
        if all(w is None for w in wyniki):
            for a in alt:
                role_bez_straznika |= {r for r in a if r != 'anonim' and r in mapa['role']
                                       and not mapa['role'][r] and r not in mapa['globalne']}
            continue
        oczek = ' albo '.join(' + '.join(a) for a in alt)
        tokeny = sorted({t for a in alt for r in a for t in mapa['role'].get(r, [])})
        znal.append(dict(plik=tr['plik'], linia=tr['linia'], regula='straznik-niezgodny', waga='blokuje',
                         odcisk_tresci=odcisk(k + '|' + wiersz['role']),
                         opis=f'`{tr["metoda"]} {tr["sciezka"]}`: mapa mówi „{oczek}", a w definicji trasy nie ma '
                              f'strażnika tej roli ({", ".join(f"`{t}`" for t in tokeny[:4])}) — dodaj go albo '
                              f'popraw rolę w mapie, jeśli dostęp ma być szerszy'))

    for p, lista in nieustalone.items():
        znal.append(dict(plik=p, linia=None, regula='rola-nieustalona', waga='ostrzega',
                         odcisk_tresci=odcisk(p + '|' + '|'.join(sorted(lista))),
                         opis=f'{len(lista)} tras z rolą „?" w mapie ({", ".join(lista[:4])}'
                              f'{" …" if len(lista) > 4 else ""}) — publiczna (anonim) czy brakuje strażnika?'))
    for r in sorted(nieznane_role):
        znal.append(dict(plik=T.MAPA, linia=None, regula='rola-nieznana', waga='ostrzega', odcisk_pliku=False,
                         odcisk_tresci=odcisk('rola|' + r),
                         opis=f'rola `{r}` użyta w ## Trasy nie ma wiersza w ## Role'))
    for r in sorted(role_bez_straznika):
        znal.append(dict(plik=T.MAPA, linia=None, regula='rola-bez-straznika', waga='ostrzega', odcisk_pliku=False,
                         odcisk_tresci=odcisk('bez-straznika|' + r),
                         opis=f'rola `{r}` nie ma „Strażnika w kodzie" — trasy tej roli nie są sprawdzane; wpisz '
                              f'nazwę z kodu (np. `Depends(get_current_user)`) albo `(globalny)`'))

    for k, tr in sorted(przedtem.items(), key=lambda x: x[0]):
        if k not in teraz and k in mapa['trasy']:
            znal.append(dict(plik=tr['plik'], linia=None, regula='trasa-zniknela', waga='ostrzega',
                             odcisk_tresci=odcisk('znikla|' + k),
                             opis=f'trasy `{tr["metoda"]} {tr["sciezka"]}` nie ma już w kodzie — usuń jej wiersz z mapy'))

    # Wyjścia: tylko linie nowe względem punktu odniesienia
    for p in pliki:
        nowe = nowe_linie(stare_tresci[p], nowe_tresci[p])
        if not nowe:
            continue
        hosty, envy, baza, bloby = {}, {}, [], []
        for nr, w in nowe:
            if KOMENTARZ.match(w):
                continue
            if T.plik_kodu(p):
                for m in HOST.finditer(w):
                    h = m.group(1).lower()
                    if not HOST_POMIJANY.search(h) and h not in mapa['hosty'] \
                            and not any(h.endswith('.' + x) for x in mapa['hosty']):
                        hosty.setdefault(h, nr)
                for m in ENV.finditer(w):
                    n = next(g for g in m.groups() if g)
                    if SEKRETNA_NAZWA.search(n) and n not in mapa['env']:
                        envy.setdefault(n, nr)
                if BLOB.search(w) and 'data:image/' not in w:
                    bloby.append(nr)
            if T.MIGRACJA.search(p) and BAZA.search(w):
                norm = re.sub(r'\s+', ' ', w.strip()).lower()
                if not any(b in norm for b in mapa['baza']):
                    baza.append((nr, w.strip()[:80]))
        for h, nr in hosty.items():
            znal.append(dict(plik=p, linia=nr, regula='host-poza-mapa', waga='ostrzega', odcisk_tresci=odcisk('host|' + h),
                             opis=f'nowy host wychodzący `{h}` bez wiersza `host` w ## Wyjścia — po co kod tam '
                                  f'wysyła dane? (eksfiltracja, SSRF, instrukcja z changes/)'))
        for n, nr in envy.items():
            znal.append(dict(plik=p, linia=nr, regula='sekret-poza-mapa', waga='ostrzega', odcisk_tresci=odcisk('env|' + n),
                             opis=f'nowy sekret ze środowiska `{n}` bez wiersza `env` w ## Wyjścia'))
        for nr, frag in baza:
            znal.append(dict(plik=p, linia=nr, regula='uprawnienie-bazy', waga='ostrzega',
                             odcisk_tresci=odcisk('baza|' + frag.lower()),
                             opis=f'nowe uprawnienie w bazie („{frag}") bez wiersza `baza` w ## Wyjścia — kto dostaje '
                                  f'dostęp i dlaczego'))
        if bloby:
            znal.append(dict(plik=p, linia=bloby[0], regula='blob-zakodowany', waga='ostrzega',
                             odcisk_tresci=odcisk('blob|' + p + '|' + ','.join(map(str, bloby))),
                             opis=f'{len(bloby)} literał(ów) base64/hex dłuższych niż 200 znaków w nowych liniach — '
                                  f'co to jest? (ukryty ładunek, osadzony klucz, dane, które powinny być plikiem)'))

    for z in znal[:MAX_ZNALEZISK]:
        wypisz(**z)
    reszta = znal[MAX_ZNALEZISK:]
    if reszta:
        blok = [z for z in reszta if z['waga'] == 'blokuje']
        wypisz(plik=reszta[0]['plik'], linia=None, regula=blok[0]['regula'] if blok else 'licznik',
               waga='blokuje' if blok else 'ostrzega', odcisk_pliku=False, odcisk_tresci='licznik',
               opis=f'+{len(reszta)} dalszych znalezisk ({len(blok)} blokujących) — pełna lista: '
                    f'python3 ralph-kontrole/ralph-kontrole.py --punkt {punkt}')
    return 0


if __name__ == '__main__':
    sys.exit(main())
