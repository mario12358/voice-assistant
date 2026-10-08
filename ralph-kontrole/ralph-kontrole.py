#!/usr/bin/env python3
"""
Ralph — kontrole jakości. Runner modułów z `ralph-kontrole/<nazwa>/`.

Kontrola to sprawdzenie, które nie zależy od tego, czy model pamięta, żeby je
uruchomić: odpala się samo w PUNKCIE WYZWALANIA — przy operacji gita, którą
Ralph i tak wykonuje.

  commit   przed `git commit`               (hook PreToolUse)
  faza     przed `git tag ralph/faza-*`     (hook PreToolUse)
  wydanie  przed `git tag … vX.Y.Z`         (hook PreToolUse)
  ci       w pipeline PR/MR                 (ręcznie: --punkt ci)

Kontrola z wagą `blokuje` zatrzymuje operację (deny — powód widzi Claude),
`ostrzega` tylko trafia do logu. Inaczej niż sędzia uprawnień ten hook CELOWO
dokłada blokady — dlatego jest osobnym modułem z osobnym przełącznikiem
(`## Kontrole` → `Włączone: tak`).

Moduł kontroli (`ralph-kontrole/<nazwa>/`):
  KONTROLA.md  frontmatter: rodzaj, uruchom, sprawdz, kiedy, zakres, przy_bledzie
  run.py       czyta listę plików ze stdin (po jednej ścieżce względnej w linii),
               zmienne RALPH_PUNKT / RALPH_ROOT; wypisuje znaleziska jako JSON
               w liniach: {plik, linia, regula, opis, waga?, odcisk_tresci?}.
               Kod 0 = sprawdzono (znaleziska mogą być), 3 = brak narzędzia.
               Treści sekretu NIGDY nie wypisuje — tylko jej odcisk.

Znalezisko uznane przez człowieka za „nie dotyczy" trafia do
`ralph/KONTROLE_WYJATKI.md` (linia z odciskiem w backtickach). Zapis do tego
pliku, do modułów kontroli i do sekcji `## Kontrole` w configu zawsze pyta
człowieka — autor kodu nie zamyka sam znalezisk, które go blokują.

Dowód i status na PR (`Status na PR: tak`, integracja z repozytorium):
  Hook PreToolUse zapisuje wynik kontroli punktu `commit` jako oczekujący dowód;
  hook PostToolUse po udanym `git commit` przypisuje go do SHA nowego commita
  (`.git/ralph-kontrole/<sha>.json` — dane tej maszyny, nic do commitowania), a po
  udanym `git push` na gałąź zadania zbiera dowody commitów `origin/main..HEAD`
  i wystawia na czubku gałęzi status commita „Ralph: kontrole (lokalnie)" przez
  `gh api`. Commit bez dowodu (zrobiony poza hookiem) = status czerwony: brak
  dowodu nie jest zielenią. To oświadczenie z tej maszyny, nie weryfikacja —
  stąd „lokalnie" w nazwie. Po rebase SHA się zmienia; dowód odnajduje się po
  `git patch-id`, bo treść zmiany została ta sama.

Tryby:
  --pretooluse          hook: JSON na stdin; Bash → punkty z komendy gita,
                        Edit/Write → ochrona plików kontroli
  --posttooluse         hook: po `git commit` dowód → SHA, po `git push` status na PR
  --punkt P [--od REF]  ręcznie / w CI; kod wyjścia 1, gdy coś blokuje
  --waliduj             linie E:/W: dla walidatora configu w ralph-start.sh
"""
import datetime
import hashlib
import json
import os
import re
import shlex
import subprocess
import sys

PUNKTY = ('commit', 'faza', 'wydanie', 'ci')
KATALOG = 'ralph-kontrole'
WYJATKI = 'ralph/KONTROLE_WYJATKI.md'
LOG = 'ralph/KONTROLE.jsonl'
CONFIG = 'ralph/config.md'
# Pliki, których skanowanie nic nie daje, a kosztuje: lockfile'y i zminifikowane.
POMIJANE = re.compile(r'(^|/)(package-lock\.json|yarn\.lock|pnpm-lock\.yaml|poetry\.lock|'
                      r'Cargo\.lock|go\.sum|deno\.lock|composer\.lock|Gemfile\.lock)$|\.min\.(js|css)$')
LIMIT_DOMYSLNY = {'commit': 20, 'faza': 300, 'wydanie': 300, 'ci': 600}
MAX_BAJTOW = 1_000_000
KONTEKST_STATUSU = 'Ralph: kontrole (lokalnie)'
DOWODY = 'ralph-kontrole'          # katalog w .git/ (git rev-parse --git-path)
OCZEKUJACY = 'oczekuje.json'
MAX_OPIS_STATUSU = 140             # limit GitHuba na description statusu


# --- Konfiguracja -------------------------------------------------------------

def bez_komentarzy(t):
    return re.sub(r'<!--.*?-->', '', t, flags=re.S)


def wczytaj_config(root):
    """{'wlaczone': bool, 'kontrole': {nazwa: {pole: wartość}}} — sekcja ## Kontrole."""
    try:
        with open(os.path.join(root, CONFIG), encoding='utf-8') as f:
            tekst = f.read()
    except OSError:
        return {'wlaczone': False, 'kontrole': {}}
    m = re.search(r'^## Kontrole[ \t]*\n(.*?)(?=^## |\Z)', tekst, re.M | re.S)
    if not m:
        return {'wlaczone': False, 'kontrole': {}}
    sekcja = bez_komentarzy(m.group(1))
    glowa, *reszta = re.split(r'^### ', sekcja, flags=re.M)

    def pola(blok):
        return {k.strip(): v.strip() for k, v in
                re.findall(r'^- \*\*(.+?)\*\*:[ \t]*(.*?)[ \t]*$', blok, re.M)}

    kontrole = {}
    for blok in reszta:
        nazwa, _, cialo = blok.partition('\n')
        nazwa = nazwa.strip().strip('`')
        if nazwa:
            kontrole[nazwa] = pola(cialo)
    wl = pola(glowa).get('Włączone', 'nie').lower() in ('tak', 'true', 'yes')
    return {'wlaczone': wl, 'kontrole': kontrole}


def pola_sekcji(root, sekcja):
    """{pole: wartość} z sekcji `## <sekcja>` configu (bez komentarzy); {} gdy brak."""
    try:
        with open(os.path.join(root, CONFIG), encoding='utf-8') as f:
            tekst = f.read()
    except OSError:
        return {}
    m = re.search(r'^## ' + re.escape(sekcja) + r'[ \t]*\n(.*?)(?=^## |\Z)', tekst, re.M | re.S)
    if not m:
        return {}
    blok = bez_komentarzy(m.group(1)).split('\n### ')[0]
    return {k.strip(): v.strip() for k, v in
            re.findall(r'^- \*\*(.+?)\*\*:[ \t]*(.*?)[ \t]*$', blok, re.M)}


def status_na_pr(root):
    """Czy wystawiać status commita: pole `Status na PR` (domyślnie tak) i integracja
    z repozytorium (`## Integracje` → `Repozytorium` ≠ brak)."""
    k = pola_sekcji(root, 'Kontrole')
    if k.get('Status na PR', 'tak').lower() not in ('tak', 'true', 'yes'):
        return False
    return pola_sekcji(root, 'Integracje').get('Repozytorium', 'brak').lower() not in ('', 'brak')


def wczytaj_modul(root, nazwa):
    """Frontmatter KONTROLA.md albo None, gdy modułu nie ma."""
    sciezka = os.path.join(root, KATALOG, nazwa, 'KONTROLA.md')
    try:
        with open(sciezka, encoding='utf-8') as f:
            tekst = f.read()
    except OSError:
        return None
    m = re.match(r'^---\n(.*?)\n---', tekst, re.S)
    meta = {}
    for linia in (m.group(1) if m else '').splitlines():
        k, sep, v = linia.partition(':')
        if sep:
            meta[k.strip()] = v.strip()
    meta['katalog'] = os.path.dirname(sciezka)
    return meta


def lista_wartosci(s):
    return [x.strip().lower() for x in re.split(r'[,\s]+', s or '') if x.strip()]


def ustawienia(root, nazwa, pola_cfg):
    """Pola z configu nadpisują domyślne z modułu."""
    modul = wczytaj_modul(root, nazwa)
    if modul is None:
        return None
    kiedy = lista_wartosci(pola_cfg.get('Kiedy') or modul.get('kiedy'))
    przy = (pola_cfg.get('Przy błędzie') or modul.get('przy_bledzie') or 'blokuje').lower()
    zakres = (pola_cfg.get('Zakres') or modul.get('zakres') or 'zmienione').lower()
    limit = pola_cfg.get('Limit czasu') or ''
    lm = re.match(r'^(\d+)\s*(s|sek|min)?', limit)
    sek = (int(lm.group(1)) * (60 if lm.group(2) == 'min' else 1)) if lm else None
    return {'nazwa': nazwa, 'modul': modul, 'kiedy': kiedy, 'przy': przy,
            'zakres': zakres, 'limit': sek}


# --- Waliduj ------------------------------------------------------------------

def waliduj(root):
    cfg = wczytaj_config(root)
    if not cfg['wlaczone']:
        return []
    out = []
    if not cfg['kontrole']:
        out.append('W:## Kontrole: Włączone: tak, ale żadnej kontroli (### <nazwa>) — nic się nie uruchomi')
    k = pola_sekcji(root, 'Kontrole')
    sp = k.get('Status na PR', 'tak').lower()
    if sp not in ('tak', 'nie', 'true', 'false', 'yes', 'no'):
        out.append(f'E:## Kontrole → Status na PR = {sp} — dozwolone: tak, nie')
    elif status_na_pr(root):
        integ = pola_sekcji(root, 'Integracje')
        if not gh_dostepny():
            out.append('W:## Kontrole → Status na PR: tak, ale nie ma `gh` w PATH — status nie będzie wystawiany')
        if KONTEKST_STATUSU not in (integ.get('Wymagane checki') or ''):
            out.append(f'W:## Integracje → Wymagane checki: brak „{KONTEKST_STATUSU}" — sędzia zmerguje PR '
                       f'bez dowodu kontroli; dopisz tę nazwę (kreator robi to sam)')
    for nazwa, pola_cfg in cfg['kontrole'].items():
        u = ustawienia(root, nazwa, pola_cfg)
        if u is None:
            dost = sorted(d for d in os.listdir(os.path.join(root, KATALOG))
                          if os.path.isfile(os.path.join(root, KATALOG, d, 'KONTROLA.md'))) \
                if os.path.isdir(os.path.join(root, KATALOG)) else []
            out.append(f'E:## Kontrole → {nazwa}: nie ma modułu {KATALOG}/{nazwa}/ '
                       f'(dostępne: {", ".join(dost) or "brak"})')
            continue
        zle = [k for k in u['kiedy'] if k not in PUNKTY]
        if zle:
            out.append(f'E:## Kontrole → {nazwa}: Kiedy = {", ".join(zle)} — dozwolone: {", ".join(PUNKTY)}')
        if not u['kiedy']:
            out.append(f'W:## Kontrole → {nazwa}: puste „Kiedy" — kontrola nigdy się nie uruchomi')
        if u['przy'] not in ('blokuje', 'ostrzega'):
            out.append(f'E:## Kontrole → {nazwa}: Przy błędzie = {u["przy"]} — dozwolone: blokuje, ostrzega')
        if u['zakres'] not in ('zmienione', 'całość', 'calosc'):
            out.append(f'E:## Kontrole → {nazwa}: Zakres = {u["zakres"]} — dozwolone: zmienione, całość')
        if u['modul'].get('rodzaj') == 'agent' and 'commit' in u['kiedy']:
            out.append(f'E:## Kontrole → {nazwa}: kontrola rodzaju agent nie może działać przy commit '
                       f'(minuty i tokeny na każdy commit) — użyj faza / wydanie')
        sprawdz = u['modul'].get('sprawdz')
        if sprawdz:
            try:
                r = subprocess.run(shlex.split(sprawdz), cwd=u['modul']['katalog'],
                                   capture_output=True, text=True, timeout=15)
                info = (r.stdout.strip() or r.stderr.strip()).splitlines()[:1]
                info = info[0] if info else ''
                if r.returncode == 3:
                    out.append(f'E:## Kontrole → {nazwa}: {info or "brak wymaganego narzędzia"}')
                elif r.returncode != 0:
                    out.append(f'E:## Kontrole → {nazwa}: sprawdzenie modułu nie przeszło ({info or r.returncode})')
                elif info.startswith('uwaga:'):
                    out.append(f'W:## Kontrole → {nazwa}: {info[6:].strip()}')
            except (OSError, subprocess.TimeoutExpired) as e:
                out.append(f'E:## Kontrole → {nazwa}: nie da się uruchomić sprawdzenia modułu ({e})')
    return out


# --- Git ------------------------------------------------------------------------

def git(root, *args):
    try:
        r = subprocess.run(['git', '-C', root, *args], capture_output=True, text=True, timeout=30)
    except (OSError, subprocess.TimeoutExpired):
        return None
    return r.stdout if r.returncode == 0 else None


def linie(s):
    return [x for x in (s or '').splitlines() if x.strip()]


def sha(root, ref):
    r = git(root, 'rev-parse', '--verify', '--quiet', ref + '^{commit}')
    return r.strip() if r else ''


def gh_dostepny():
    import shutil
    return bool(os.environ.get('RALPH_GH') or shutil.which('gh'))


# --- Dowody per commit -------------------------------------------------------------

def katalog_dowodow(root):
    p = git(root, 'rev-parse', '--git-path', DOWODY)
    if not p:
        return None
    p = p.strip()
    if not os.path.isabs(p):
        p = os.path.join(root, p)
    try:
        os.makedirs(p, exist_ok=True)
    except OSError:
        return None
    return p


def zapisz_json(sciezka, dane):
    try:
        with open(sciezka, 'w', encoding='utf-8') as f:
            json.dump(dane, f, ensure_ascii=False, indent=1)
        return True
    except OSError:
        return False


def wczytaj_json(sciezka):
    try:
        with open(sciezka, encoding='utf-8') as f:
            return json.load(f)
    except (OSError, ValueError):
        return None


def podsumowanie_kontroli(root, punkt, log, bledy):
    """Co z przebiegu zostaje w dowodzie: per kontrola najgorsza decyzja i liczby, bez
    treści znalezisk (dowód trafia w opis statusu, nie w log)."""
    cfg = wczytaj_config(root)
    skonfigurowane = [n for n, p in cfg['kontrole'].items()
                      if (u := ustawienia(root, n, p)) and punkt in u['kiedy']]
    kontrole = {}
    for w in log:
        k = kontrole.setdefault(w['kontrola'], {'decyzja': 'czysto', 'ostrzezen': 0, 'wyjatkow': 0})
        d = w.get('decyzja')
        if d == 'blad':
            k['decyzja'] = 'blad'
            k['opis'] = w.get('opis', '')
        elif d == 'ostrzega':
            k['ostrzezen'] += 1
            if k['decyzja'] == 'czysto':
                k['decyzja'] = 'ostrzega'
        elif d == 'wyjatek':
            k['wyjatkow'] += 1
    for n in skonfigurowane:
        kontrole.setdefault(n, {'decyzja': 'blad', 'ostrzezen': 0, 'wyjatkow': 0, 'opis': 'nie uruchomiono'})
    return {'punkt': punkt, 'kontrole': kontrole, 'skonfigurowane': skonfigurowane}


def zapisz_oczekujacy(root, log, bledy):
    """PreToolUse przed commitem: wynik kontroli czeka na SHA, który powstanie za chwilę."""
    kat = katalog_dowodow(root)
    if not kat:
        return
    head = sha(root, 'HEAD')
    dane = podsumowanie_kontroli(root, 'commit', log, bledy)
    dane.update({'head': head, 'rodzic': sha(root, 'HEAD~1') if head else '',
                 'ts': datetime.datetime.now().isoformat(timespec='seconds')})
    zapisz_json(os.path.join(kat, OCZEKUJACY), dane)


def patch_id(root, commit):
    try:
        a = subprocess.run(['git', '-C', root, 'diff-tree', '-p', '--no-commit-id', '--root', commit],
                           capture_output=True, text=True, timeout=30)
        b = subprocess.run(['git', '-C', root, 'patch-id', '--stable'], input=a.stdout,
                           capture_output=True, text=True, timeout=30)
    except (OSError, subprocess.TimeoutExpired):
        return ''
    return b.stdout.split()[0] if b.stdout.split() else ''


def dowod_po_commicie(root, cmd):
    """PostToolUse po `git commit`: oczekujący dowód → plik <sha>.json, jeśli nowy HEAD
    wyrósł z tego HEAD-a, który hook widział (albo z jego rodzica przy --amend)."""
    kat = katalog_dowodow(root)
    if not kat:
        return None
    ocz = wczytaj_json(os.path.join(kat, OCZEKUJACY))
    if not ocz:
        return None
    nowy = sha(root, 'HEAD')
    if not nowy or nowy == ocz.get('head'):
        return None                      # commit się nie wykonał — dowód dalej czeka
    rodzic = sha(root, 'HEAD~1')
    amend = bool(re.search(r'--amend\b', cmd))
    if not (rodzic == ocz.get('head') or (amend and rodzic == ocz.get('rodzic'))):
        return None                      # HEAD zmienił się inaczej (checkout, pull) — to nie ten commit
    dowod = {'sha': nowy, 'rodzic': rodzic, 'patch_id': patch_id(root, nowy), 'ts': ocz.get('ts'),
             'kontrole': ocz.get('kontrole', {}), 'skonfigurowane': ocz.get('skonfigurowane', [])}
    zapisz_json(os.path.join(kat, nowy + '.json'), dowod)
    try:
        os.remove(os.path.join(kat, OCZEKUJACY))
    except OSError:
        pass
    return dowod


def dowod_commita(root, kat, commit):
    """Dowód po SHA, a gdy go nie ma — po patch-id (rebase/squash zmienia SHA, nie treść)."""
    d = wczytaj_json(os.path.join(kat, commit + '.json'))
    if d:
        return d
    pid = patch_id(root, commit)
    if not pid:
        return None
    for nazwa in os.listdir(kat):
        if not nazwa.endswith('.json') or nazwa == OCZEKUJACY:
            continue
        d = wczytaj_json(os.path.join(kat, nazwa))
        if d and d.get('patch_id') == pid:
            d = dict(d, sha=commit, przepisany_z=d.get('sha'))
            zapisz_json(os.path.join(kat, commit + '.json'), d)
            return d
    return None


# --- Status na PR ------------------------------------------------------------------

def galaz_z_pushu(cmd):
    """Nazwa gałęzi z członu `git push …` komendy albo '' (push bez argumentów); None, gdy
    komenda nie pushuje."""
    czl = czlony_komendy(cmd) or []
    for tokeny in czl:
        pk = podkomenda_gita(tokeny)
        if not pk or pk[0] != 'push':
            continue
        args = [a for a in pk[1] if not a.startswith('-')]
        if len(args) >= 2:
            return args[1].split(':')[-1].replace('refs/heads/', '')
        return ''
    if re.search(r'\bgit\b[^;&|\n]*\bpush\b', cmd):
        return ''
    return None


def repo_github(root):
    r = pola_sekcji(root, 'Integracje').get('Repo', '')
    if re.match(r'^[\w.-]+/[\w.-]+$', r):
        return r
    url = (git(root, 'remote', 'get-url', 'origin') or '').strip()
    m = re.search(r'github\.com[:/]([^/]+/[^/]+?)(?:\.git)?/?$', url)
    return m.group(1) if m else ''


def galaz_bazowa(root):
    for ref in ('origin/main', 'origin/master'):
        if sha(root, ref):
            return ref
    h = (git(root, 'symbolic-ref', '--quiet', 'refs/remotes/origin/HEAD') or '').strip()
    return h.replace('refs/remotes/', '') if h else 'origin/main'


def ostatni_audyt(root):
    """(data, decyzja) ostatniego przebiegu audytu zależności z logu — jest w opisie
    statusu, bo audyt biegnie przy tagach, nie przy commicie, i bez daty „czysto" nic nie mówi."""
    try:
        with open(os.path.join(root, LOG), encoding='utf-8') as f:
            wiersze = f.read().splitlines()
    except OSError:
        return None
    for linia in reversed(wiersze[-5000:]):
        try:
            w = json.loads(linia)
        except ValueError:
            continue
        if w.get('kontrola') == 'audyt-zaleznosci' and w.get('decyzja') in ('czysto', 'ostrzega', 'blokuje', 'wyjatek'):
            return (w.get('ts', '')[:10], w['decyzja'])
    return None


def zbierz_status(root, kat, commity):
    """(stan, opis) statusu dla listy commitów gałęzi (od najstarszego)."""
    bez_dowodu, nie_wykonane, kontrole, ostrzezen, ts = [], [], set(), 0, ''
    for c in commity:
        d = dowod_commita(root, kat, c)
        if not d:
            bez_dowodu.append(c[:7])
            continue
        ts = max(ts, d.get('ts') or '')
        for n, k in (d.get('kontrole') or {}).items():
            kontrole.add(n)
            ostrzezen += k.get('ostrzezen', 0)
            if k.get('decyzja') == 'blad':
                nie_wykonane.append(f'{n}@{c[:7]}')
    audyt = ostatni_audyt(root)
    a_txt = f' · audyt OSV {audyt[0]} {audyt[1]}' if audyt else ''
    if bez_dowodu:
        opis = (f'brak dowodu kontroli dla {len(bez_dowodu)} z {len(commity)} commitów '
                f'({", ".join(bez_dowodu[:3])}{"…" if len(bez_dowodu) > 3 else ""}) — commit poza hookiem')
        return 'failure', opis[:MAX_OPIS_STATUSU]
    if nie_wykonane:
        opis = f'kontrola nie wykonała się: {", ".join(nie_wykonane[:3])}{"…" if len(nie_wykonane) > 3 else ""}'
        return 'failure', opis[:MAX_OPIS_STATUSU]
    kiedy = ts.replace('T', ' ')[:16]
    n_c = f'{len(commity)} commit' + ('' if len(commity) == 1 else 'y' if 2 <= len(commity) <= 4 else 'ów')
    wynik = 'czysto' if not ostrzezen else f'{ostrzezen} ostrz.'
    lista = ', '.join(sorted(kontrole)) or 'brak kontroli'
    opis = f'{wynik}: {lista} · {n_c} · {kiedy}{a_txt}'
    if len(opis) > MAX_OPIS_STATUSU:
        opis = f'{wynik}: {len(kontrole)} kontrole · {n_c} · {kiedy}{a_txt}'
    return 'success', opis[:MAX_OPIS_STATUSU]


def wystaw_status(root, repo, commit, stan, opis):
    """POST /repos/{repo}/statuses/{sha} przez gh (token z `gh auth login`). Zwraca błąd albo None."""
    gh = os.environ.get('RALPH_GH') or 'gh'
    try:
        r = subprocess.run([gh, 'api', f'repos/{repo}/statuses/{commit}', '-f', f'state={stan}',
                            '-f', f'context={KONTEKST_STATUSU}', '-f', f'description={opis}', '--silent'],
                           capture_output=True, text=True, timeout=25, cwd=root)
    except FileNotFoundError:
        return 'brak gh w PATH'
    except (OSError, subprocess.TimeoutExpired) as e:
        return f'gh api: {e}'
    if r.returncode != 0:
        return 'gh api: ' + ' '.join((r.stderr or r.stdout).strip().splitlines()[-2:])[:200]
    return None


def status_po_pushu(root, cmd):
    """PostToolUse po `git push`: dowody commitów gałęzi → jeden status na jej czubku.
    Zwraca tekst dla Claude'a albo None (nie dotyczy)."""
    galaz = galaz_z_pushu(cmd)
    if galaz is None or not status_na_pr(root):
        return None
    if not galaz:
        galaz = (git(root, 'branch', '--show-current') or '').strip()
    if not galaz or galaz in ('main', 'master'):
        return None
    lokalny, zdalny = sha(root, galaz), sha(root, 'refs/remotes/origin/' + galaz)
    if not lokalny or lokalny != zdalny:
        return None                      # push się nie udał albo poszedł gdzie indziej
    baza = galaz_bazowa(root)
    commity = linie(git(root, 'rev-list', '--reverse', f'{baza}..{galaz}'))
    if not commity:
        return None
    kat = katalog_dowodow(root)
    if not kat:
        return None
    stan, opis = zbierz_status(root, kat, commity)
    repo = repo_github(root)
    ts = datetime.datetime.now().isoformat(timespec='seconds')
    wpis = {'ts': ts, 'punkt': 'push', 'head': lokalny[:7], 'kontrola': 'status-pr', 'galaz': galaz,
            'stan': stan, 'opis': opis, 'commitow': len(commity)}
    if not repo:
        wpis['decyzja'] = 'blad'
        wpis['blad'] = 'nie znam repozytorium (## Integracje → Repo ani remote origin)'
    else:
        blad = wystaw_status(root, repo, lokalny, stan, opis)
        wpis['decyzja'] = 'blad' if blad else 'wystawiony'
        if blad:
            wpis['blad'] = blad
    zapisz_log(root, [wpis])
    if wpis['decyzja'] == 'blad':
        return (f'Ralph — status „{KONTEKST_STATUSU}" na {galaz}@{lokalny[:7]} NIE został wystawiony '
                f'({wpis["blad"]}). Treść: {stan} — {opis}. Powiedz o tym człowiekowi przy najbliższym raporcie.')
    return f'Ralph — status „{KONTEKST_STATUSU}" na {galaz}@{lokalny[:7]}: {stan} — {opis}'


def ostatni_tag(root, wzorzec):
    t = git(root, 'describe', '--tags', '--abbrev=0', '--match', wzorzec, 'HEAD')
    return t.strip() if t else None


def pliki_zakresu(root, punkt, zakres, plan_commit=None, od=None):
    """Lista ścieżek (względnych) do sprawdzenia."""
    if zakres in ('całość', 'calosc'):
        return linie(git(root, 'ls-files'))
    if punkt == 'commit':
        p = plan_commit or {}
        zbior = set(linie(git(root, 'diff', '--cached', '--name-only', '--diff-filter=ACMR')))
        if p.get('wszystkie_sledzone'):
            zbior |= set(linie(git(root, 'diff', '--name-only', '--diff-filter=ACMR', 'HEAD')))
        for sc in p.get('dodawane', []):
            if sc in ('.', '-A', '--all', ':/'):
                zbior |= set(linie(git(root, 'diff', '--name-only', '--diff-filter=ACMR', 'HEAD')))
                zbior |= set(linie(git(root, 'ls-files', '--others', '--exclude-standard')))
                continue
            pelna = os.path.join(root, sc)
            if os.path.isdir(pelna):
                zbior |= set(linie(git(root, 'ls-files', '--modified', '--others',
                                       '--exclude-standard', '--', sc)))
            elif os.path.isfile(pelna):
                zbior.add(os.path.relpath(pelna, root))
        return sorted(zbior)
    if punkt == 'faza':
        od = od or ostatni_tag(root, 'ralph/faza-*')
    elif punkt == 'wydanie':
        od = od or ostatni_tag(root, 'v[0-9]*')
    if not od:
        return linie(git(root, 'ls-files'))
    return linie(git(root, 'diff', '--name-only', '--diff-filter=ACMR', f'{od}...HEAD'))


# Pliki frameworka w projekcie, nie kod projektu: hook sędziego w .claude/hooks/ aktualizuje się
# przy starcie i wchodzi do pierwszego commita sesji — sast blokował go za celowe `shell=True`
POMIJANE_KATALOGI = (KATALOG + '/', '.claude/')


def odfiltruj(root, pliki):
    out = []
    for p in pliki:
        if p.startswith(POMIJANE_KATALOGI) or POMIJANE.search(p):
            continue
        pelna = os.path.join(root, p)
        try:
            if not os.path.isfile(pelna) or os.path.getsize(pelna) > MAX_BAJTOW:
                continue
        except OSError:
            continue
        out.append(p)
    return out


# --- Uruchomienie kontroli ---------------------------------------------------------

def odcisk(nazwa, z):
    # `odcisk_pliku: false` — znalezisko dotyczy rzeczy, nie miejsca (jedno osłabienie
    # oznaczone w kilku plikach); przeniesienie znacznika do innego pliku nie unieważnia wyjątku
    plik = z.get('plik', '') if z.get('odcisk_pliku', True) else ''
    baza = '|'.join([nazwa, z.get('regula', ''), plik,
                     z.get('odcisk_tresci') or z.get('opis', '')])
    return hashlib.sha1(baza.encode('utf-8')).hexdigest()[:12]


def wczytaj_wyjatki(root):
    try:
        with open(os.path.join(root, WYJATKI), encoding='utf-8') as f:
            return set(re.findall(r'^\s*-\s*`([0-9a-f]{12})`', f.read(), re.M))
    except OSError:
        return set()


def uruchom_kontrole(root, u, punkt, pliki, budzet, od=None):
    """Zwraca (znaleziska, błąd albo None)."""
    uruchom = u['modul'].get('uruchom')
    if not uruchom:
        return [], 'moduł bez pola „uruchom"'
    limit = min(u['limit'] or LIMIT_DOMYSLNY[punkt], max(budzet, 1))
    env = {**os.environ, 'RALPH_PUNKT': punkt, 'RALPH_ROOT': root}
    if od:
        env['RALPH_OD'] = od      # punkt odniesienia z `--od` (CI) — moduły liczące „nowe od…" go potrzebują
    try:
        r = subprocess.run(shlex.split(uruchom), cwd=u['modul']['katalog'], env=env,
                           input='\n'.join(pliki) + '\n', capture_output=True, text=True,
                           timeout=limit)
    except subprocess.TimeoutExpired:
        return [], f'przekroczony limit czasu ({limit} s)'
    except OSError as e:
        return [], f'nie da się uruchomić: {e}'
    if r.returncode == 3:
        return [], (r.stderr.strip().splitlines() or ['brak wymaganego narzędzia'])[0]
    if r.returncode != 0:
        return [], f'kod {r.returncode}: ' + ' '.join(r.stderr.strip().splitlines()[-2:])
    out = []
    for linia in r.stdout.splitlines():
        try:
            z = json.loads(linia)
        except ValueError:
            continue
        if isinstance(z, dict) and z.get('plik') is not None:
            out.append(z)
    return out, None


def zapisz_log(root, wpisy):
    if not wpisy:
        return
    try:
        sciezka = os.path.join(root, LOG)
        os.makedirs(os.path.dirname(sciezka), exist_ok=True)
        with open(sciezka, 'a', encoding='utf-8') as f:
            for w in wpisy:
                f.write(json.dumps(w, ensure_ascii=False) + '\n')
    except OSError:
        pass    # log nigdy nie może zablokować pracy


def przebieg(root, punkt, plan_commit=None, od=None):
    """Uruchamia kontrole przypisane do punktu. Zwraca (blokujące, ostrzeżenia, błędy)."""
    cfg = wczytaj_config(root)
    if not cfg['wlaczone']:
        return [], [], []
    wyjatki = wczytaj_wyjatki(root)
    ts = datetime.datetime.now().isoformat(timespec='seconds')
    head = (git(root, 'rev-parse', '--short', 'HEAD') or '').strip()
    blok, ostrz, bledy, log = [], [], [], []
    budzet = 570 if punkt != 'commit' else 120   # hook ma 600 s; zapas na resztę
    start = datetime.datetime.now()
    for nazwa, pola_cfg in cfg['kontrole'].items():
        u = ustawienia(root, nazwa, pola_cfg)
        if u is None or punkt not in u['kiedy']:
            continue
        pliki = odfiltruj(root, pliki_zakresu(root, punkt, u['zakres'], plan_commit, od))
        zuzyte = (datetime.datetime.now() - start).total_seconds()
        znal, blad = uruchom_kontrole(root, u, punkt, pliki, budzet - zuzyte, od=od)
        if blad:
            bledy.append(f'{nazwa}: {blad}')
            log.append({'ts': ts, 'punkt': punkt, 'head': head, 'kontrola': nazwa,
                        'decyzja': 'blad', 'opis': blad})
            continue
        for z in znal:
            waga = z.get('waga') or u['przy']
            if u['przy'] == 'ostrzega':
                waga = 'ostrzega'          # config może tylko złagodzić moduł, nie zaostrzyć
            od_ = odcisk(nazwa, z)
            wpis = {'ts': ts, 'punkt': punkt, 'head': head, 'kontrola': nazwa,
                    'regula': z.get('regula', ''), 'plik': z.get('plik', ''),
                    'linia': z.get('linia'), 'opis': z.get('opis', ''), 'odcisk': od_}
            if od_ in wyjatki:
                wpis['decyzja'] = 'wyjatek'
            elif waga == 'blokuje':
                wpis['decyzja'] = 'blokuje'
                blok.append(wpis)
            else:
                wpis['decyzja'] = 'ostrzega'
                ostrz.append(wpis)
            log.append(wpis)
        if not znal:
            log.append({'ts': ts, 'punkt': punkt, 'head': head, 'kontrola': nazwa,
                        'decyzja': 'czysto', 'plikow': len(pliki)})
    zapisz_log(root, log)
    return blok, ostrz, bledy, log


def opisz(wpisy, limit=25):
    out = []
    for w in wpisy[:limit]:
        gdzie = w['plik'] + (f':{w["linia"]}' if w.get('linia') else '')
        regula = f'/{w["regula"]}' if w.get('regula') else ''
        out.append(f'  - [{w["kontrola"]}{regula}] {gdzie} — {w["opis"]} (odcisk `{w["odcisk"]}`)')
    if len(wpisy) > limit:
        out.append(f'  … i {len(wpisy) - limit} kolejnych (ralph/KONTROLE.jsonl)')
    return '\n'.join(out)


# --- Parsowanie komendy ------------------------------------------------------------

OPCJE_GITA_Z_ARGUMENTEM = {'-C', '-c', '--git-dir', '--work-tree', '--namespace', '--exec-path'}


def czlony_komendy(cmd):
    """Lista członów (listy tokenów) rozdzielonych &&, ||, ;, |, nową linią."""
    try:
        lex = shlex.shlex(cmd.replace('\n', ' ; '), posix=True, punctuation_chars=';&|')
        lex.whitespace_split = True
        tokeny = list(lex)
    except ValueError:
        return None
    out, biez = [], []
    for t in tokeny:
        if t and set(t) <= set(';&|'):
            if biez:
                out.append(biez)
            biez = []
        else:
            biez.append(t)
    if biez:
        out.append(biez)
    return out


def podkomenda_gita(tokeny):
    """(podkomenda, argumenty, katalog z -C) dla członu `git …` albo None."""
    i = 0
    while i < len(tokeny) and re.match(r'^[A-Za-z_][A-Za-z0-9_]*=', tokeny[i]):
        i += 1                                  # VAR=x git …
    if i >= len(tokeny) or os.path.basename(tokeny[i]) != 'git':
        return None
    i += 1
    katalog = None
    while i < len(tokeny) and tokeny[i].startswith('-'):
        if tokeny[i] in OPCJE_GITA_Z_ARGUMENTEM and i + 1 < len(tokeny):
            if tokeny[i] == '-C':
                katalog = tokeny[i + 1]
            i += 2
        else:
            i += 1
    if i >= len(tokeny):
        return None
    return tokeny[i], tokeny[i + 1:], katalog


WERSJA = re.compile(r'^v\d+\.\d+\.\d+')
# Człony, które mogą stać PRZED commitem w tej samej komendzie: zmieniają co najwyżej
# indeks, nie treść plików. Wszystko inne (printf >>, sed -i, python, npm) pisze do
# plików już PO tym, jak hook je przeczytał — kontrola widziałaby stan sprzed zmiany.
GIT_PRZED_COMMITEM = {'add', 'rm', 'mv', 'status', 'diff', 'log', 'show'}
# Komendy, które tylko czytają — sekcja 4 instrukcji każe zrobić `grep MUTANT` przed `git add`,
# a kontrola kształtu odrzucała właśnie to (pierwsza sesja na Specky). Zapis przez przekierowanie
# albo opcję wykonującą komendę (`find -exec`, `-delete`) dalej jest odrzucany.
ODCZYT_PRZED_COMMITEM = {'grep', 'egrep', 'fgrep', 'rg', 'ls', 'cat', 'head', 'tail', 'wc', 'find', 'echo', 'test', '['}
OPCJE_ZAPISU = {'-exec', '-execdir', '-delete', '-ok', '-okdir', '-fprint', '-fprintf', '-fls'}


def tylko_odczyt(tokeny):
    if not tokeny or os.path.basename(tokeny[0]) not in ODCZYT_PRZED_COMMITEM:
        return False
    return not any('>' in t or t in OPCJE_ZAPISU for t in tokeny)
OPAKOWANIA = {'bash', 'sh', 'zsh', 'dash', 'eval', 'xargs', 'env', 'command', 'nice', 'time',
              'nohup', 'timeout', 'watch', 'sudo', 'exec', 'script', 'parallel'}


def aliasy_gita(root):
    """{alias: rozwinięcie} z konfiguracji gita — `git ci` to też commit."""
    try:
        r = subprocess.run(['git', '-C', root, 'config', '--get-regexp', r'^alias\.'],
                           capture_output=True, text=True, timeout=10)
    except (OSError, subprocess.TimeoutExpired):
        return {}
    out = {}
    for linia in r.stdout.splitlines():
        k, _, v = linia.partition(' ')
        out[k[len('alias.'):]] = v.strip()
    return out


def podkomenda(tokeny, aliasy):
    """Jak podkomenda_gita, ale z rozwiniętym aliasem. Alias powłoki (`!…`) wspominający
    commit/tag zwraca podkomendę '!powloka' — nie da się go przeanalizować."""
    pk = podkomenda_gita(tokeny)
    if not pk:
        return None
    sub, args, kat = pk
    exp = (aliasy or {}).get(sub)
    if exp:
        if exp.startswith('!'):
            return ('!powloka' if re.search(r'\b(commit|tag)\b', exp) else sub), args, kat
        try:
            t = shlex.split(exp)
        except ValueError:
            t = []
        if t:
            return t[0], t[1:] + args, kat
    return sub, args, kat


def nazwy_tagow(args):
    if any(a in ('-d', '--delete', '-l', '--list', '-v', '--verify') for a in args):
        return []
    nazwy = [a for a in args if not a.startswith('-')]
    for flaga in ('-m', '-F', '--message', '--file'):     # argument po -m to wiadomość
        if flaga in args:
            k = args.index(flaga)
            if k + 1 < len(args) and args[k + 1] in nazwy:
                nazwy.remove(args[k + 1])
    return nazwy


def cel_czlonu(tokeny, aliasy):
    """'commit' | 'tag' (fazy/wydania) | None — czy człon uruchamia kontrole."""
    pk = podkomenda(tokeny, aliasy)
    if not pk:
        return None
    sub, args, _ = pk
    if sub in ('commit', '!powloka'):
        return 'commit'
    if sub == 'tag' and any(n.startswith('ralph/faza-') or WERSJA.match(n) for n in nazwy_tagow(args)):
        return 'tag'
    return None


def problem_kolejnosci(cmd, aliasy=None):
    """Powód, dla którego komendy nie da się rzetelnie sprawdzić, albo None.

    Hook czyta pliki PRZED wykonaniem komendy. `printf … >> plik && git add plik && git commit`
    przechodził: kontrola widziała plik bez dopisanej treści (pierwszy test na Specky)."""
    czl = czlony_komendy(cmd)
    if czl is None:
        m = re.search(r'\bgit\b[^;&|\n]*\b(commit|tag)\b', cmd)
        if not m:
            return None
        prefiks = re.sub(r'[\s;&|]+$', '', cmd[:m.start()])
        czl = czlony_komendy(prefiks) if prefiks.strip() else []
        if czl is None:
            return 'kolejnosc'
        przed = czl
    else:
        for tokeny in czl:
            i = 0
            while i < len(tokeny) and re.match(r'^[A-Za-z_][A-Za-z0-9_]*=', tokeny[i]):
                i += 1
            if i < len(tokeny) and os.path.basename(tokeny[i]) in OPAKOWANIA \
                    and re.search(r'\bgit\b.*\b(commit|tag)\b', ' '.join(tokeny[i + 1:])):
                return 'opakowanie'
            pk = podkomenda(tokeny, aliasy)
            if pk and pk[0] == '!powloka':
                return 'alias'
        # KAŻDY cel, nie tylko pierwszy: `git commit && git tag ralph/faza-N` taguje commit,
        # którego kontrola fazy nie widziała (w chwili hooka HEAD był jeszcze poprzedni)
        cele = [k for k, t in enumerate(czl) if cel_czlonu(t, aliasy)]
        if not cele:
            return None
        przed = czl[:cele[-1]]
    for tokeny in przed:
        if tokeny and tokeny[0] in ('cd', 'pushd', 'true', ':'):
            continue
        pk = podkomenda(tokeny, aliasy)
        if pk and pk[0] in GIT_PRZED_COMMITEM:
            continue
        if tylko_odczyt(tokeny):
            continue
        return 'kolejnosc'
    return None


def punkty_z_komendy(cmd, aliasy=None):
    """{punkt: plan} — co ta komenda zrobi z repozytorium. Plan commita: pliki
    dodawane w tej samej komendzie (`git add a b && git commit` — w chwili hooka
    nic jeszcze nie jest w indeksie)."""
    czl = czlony_komendy(cmd)
    if czl is None:
        # Nieparsowalna (heredoc, niedomknięty cudzysłów) — gruby regex, bez planu dodawania.
        p = {}
        if re.search(r'\bgit\b[^;&|\n]*\bcommit\b', cmd):
            p['commit'] = {'dodawane': [], 'wszystkie_sledzone': bool(re.search(r'\bcommit\b[^;&|\n]*\s-(a|-all)\b', cmd))}
        if re.search(r'\bgit\b[^;&|\n]*\btag\b[^;&|\n]*\bralph/faza-', cmd):
            p['faza'] = {}
        if re.search(r'\bgit\b[^;&|\n]*\btag\b[^;&|\n]*\sv\d+\.\d+\.\d+', cmd):
            p['wydanie'] = {}
        return p
    dodawane, punkty = [], {}
    for tokeny in czl:
        pk = podkomenda(tokeny, aliasy)
        if not pk:
            continue
        sub, args, _kat = pk
        if sub == 'add':
            for a in args:
                if a in ('-A', '--all'):
                    dodawane.append('-A')
                elif a == '--':
                    continue
                elif not a.startswith('-'):
                    dodawane.append(a)
        elif sub == 'commit':
            wszystkie = any(a in ('-a', '--all') or (re.match(r'^-[a-zA-Z]*a[a-zA-Z]*$', a) and not a.startswith('--'))
                            for a in args if a not in ('-m', '--amend'))
            punkty['commit'] = {'dodawane': list(dodawane), 'wszystkie_sledzone': wszystkie}
        elif sub == 'tag':
            nazwy = nazwy_tagow(args)
            if any(n.startswith('ralph/faza-') for n in nazwy):
                punkty['faza'] = {}
            if any(WERSJA.match(n) for n in nazwy):
                punkty['wydanie'] = {}
    return punkty


# --- Ochrona plików kontroli -------------------------------------------------------

def chroniona(root, sciezka):
    if not sciezka:
        return False
    pelna = os.path.normpath(os.path.join(root, sciezka)) if not os.path.isabs(sciezka) else os.path.normpath(sciezka)
    rel = os.path.relpath(pelna, root)
    return rel == WYJATKI or rel == KATALOG or rel.startswith(KATALOG + os.sep)


def sekcja_kontrole(tekst):
    m = re.search(r'^## Kontrole[ \t]*\n.*?(?=^## |\Z)', tekst or '', re.M | re.S)
    return m.group(0) if m else ''


def narusza_config(root, narzedzie, wejscie):
    """Czy edycja ralph/config.md zmienia sekcję ## Kontrole."""
    sciezka = wejscie.get('file_path') or ''
    if os.path.normpath(os.path.join(root, sciezka)) != os.path.normpath(os.path.join(root, CONFIG)):
        return False
    try:
        with open(os.path.join(root, CONFIG), encoding='utf-8') as f:
            obecny = f.read()
    except OSError:
        return False
    sek = sekcja_kontrole(obecny)
    if narzedzie == 'Write':
        return sekcja_kontrole(wejscie.get('content', '')) != sek
    edycje = wejscie.get('edits') or [wejscie]
    for e in edycje:
        stary = e.get('old_string') or ''
        if stary and stary in sek:
            return True
        if '## Kontrole' in (e.get('new_string') or '') or '## Kontrole' in stary:
            return True
    return False


ZAPIS_W_BASHU = re.compile(r'>|\btee\b|\bsed\s+(-[a-zA-Z]*i|--in-place)|\b(mv|cp|rm|truncate|install)\b|\bperl\s+-[a-zA-Z]*i')


def bash_rusza_chronione(cmd):
    if not re.search(re.escape(KATALOG) + r'/|KONTROLE_WYJATKI', cmd):
        return False
    return bool(ZAPIS_W_BASHU.search(cmd))


# --- Hook --------------------------------------------------------------------------

def wyjscie(decyzja, powod):
    print(json.dumps({'hookSpecificOutput': {
        'hookEventName': 'PreToolUse',
        'permissionDecision': decyzja,
        'permissionDecisionReason': powod,
    }}, ensure_ascii=False))
    sys.exit(0)


def tryb_pretooluse():
    try:
        dane = json.load(sys.stdin)
    except ValueError:
        sys.exit(0)
    root = os.environ.get('CLAUDE_PROJECT_DIR') or dane.get('cwd') or os.getcwd()
    if not wczytaj_config(root)['wlaczone']:
        sys.exit(0)
    narzedzie = dane.get('tool_name', '')
    wejscie = dane.get('tool_input') or {}

    if narzedzie in ('Edit', 'Write', 'MultiEdit', 'NotebookEdit'):
        sciezka = wejscie.get('file_path') or wejscie.get('notebook_path') or ''
        if chroniona(root, sciezka):
            wyjscie('ask', 'Ralph: zapis do plików kontroli jakości (moduły / wyjątki) — '
                           'decyzja należy do człowieka, nie do autora kodu')
        if narusza_config(root, narzedzie, wejscie):
            wyjscie('ask', 'Ralph: zmiana sekcji ## Kontrole w ralph/config.md — decyzja należy do człowieka')
        sys.exit(0)

    if narzedzie != 'Bash':
        sys.exit(0)
    cmd = wejscie.get('command') or ''
    if bash_rusza_chronione(cmd):
        wyjscie('ask', 'Ralph: komenda zmienia pliki kontroli jakości (moduły / wyjątki) — '
                       'decyzja należy do człowieka')
    aliasy = aliasy_gita(root)
    problem = problem_kolejnosci(cmd, aliasy)
    if problem:
        powody = {
            'kolejnosc': 'w tej samej komendzie przed commitem/tagiem stoi coś, co może zmienić pliki — '
                         'kontrola czyta pliki PRZED wykonaniem komendy i sprawdziłaby stan sprzed zmiany',
            'opakowanie': 'git commit/tag uruchomiony przez inną komendę (bash -c, xargs, env…) — '
                          'kontrola nie widzi, co dokładnie zostanie zacommitowane',
            'alias': 'alias gita, który jest skryptem powłoki, wywołuje commit/tag — nie da się go sprawdzić',
        }
        wyjscie('deny', f'Ralph — kontrole jakości: {powody[problem]}. Rozdziel: najpierw zmiana plików '
                        f'osobnym poleceniem, potem samo `git add <pliki> && git commit …` (tag — też '
                        f'osobnym poleceniem, po commicie). To nie jest znalezisko w kodzie, tylko kształt komendy.')
    punkty = punkty_z_komendy(cmd, aliasy)
    if not punkty:
        sys.exit(0)

    wszystkie_blok, wszystkie_ostrz, wszystkie_bledy, log_commitu = [], [], [], None
    for punkt in ('commit', 'faza', 'wydanie'):
        if punkt in punkty:
            b, o, e, lg = przebieg(root, punkt, plan_commit=punkty[punkt] if punkt == 'commit' else None)
            wszystkie_blok += b
            wszystkie_ostrz += o
            wszystkie_bledy += [f'{punkt}/{x}' for x in e]
            if punkt == 'commit':
                log_commitu = (lg, e)
    if not wszystkie_blok and log_commitu is not None:
        zapisz_oczekujacy(root, *log_commitu)   # commit zaraz powstanie — dowód czeka na jego SHA
    if wszystkie_blok:
        tekst = (f'Ralph — kontrole jakości zatrzymały tę operację ({len(wszystkie_blok)} blokujących):\n'
                 + opisz(wszystkie_blok) +
                 '\nNapraw i ponów. Jeśli znalezisko jest błędne, NIE obchodź kontroli: zgłoś '
                 'RALPH BLOCKED z odciskiem i uzasadnieniem — wyjątek wpisuje człowiek '
                 f'do {WYJATKI}.')
        if wszystkie_ostrz:
            tekst += f'\nPonadto ostrzeżenia ({len(wszystkie_ostrz)}):\n' + opisz(wszystkie_ostrz, 10)
        wyjscie('deny', tekst)
    # Bez blokady: brak decyzji — operacja idzie zwykłą ścieżką uprawnień. Ostrzeżenia
    # i błędy kontroli trafiają do Claude'a jako additionalContext (nie blokują).
    if wszystkie_ostrz or wszystkie_bledy:
        tekst = ''
        if wszystkie_ostrz:
            tekst += (f'Ralph — kontrole jakości: {len(wszystkie_ostrz)} ostrzeżeń (nie blokują — '
                      f'wagę każdej reguły ustala moduł w ralph-kontrole/<kontrola>/KONTROLA.md, '
                      f'a „Przy błędzie" w configu może ją tylko złagodzić; to zamierzone, nie defekt. '
                      f'Odnotuj w REPORT.md, jeśli dotyczą Twojej zmiany):\n' + opisz(wszystkie_ostrz, 15))
        if wszystkie_bledy:
            tekst += ('\nKontrole, które się nie wykonały (operacja przeszła BEZ nich — powiedz o tym '
                      'człowiekowi): ' + '; '.join(wszystkie_bledy))
        print(json.dumps({'hookSpecificOutput': {
            'hookEventName': 'PreToolUse',
            'additionalContext': tekst.strip(),
        }}, ensure_ascii=False))
    sys.exit(0)


def tryb_posttooluse():
    try:
        dane = json.load(sys.stdin)
    except ValueError:
        sys.exit(0)
    root = os.environ.get('CLAUDE_PROJECT_DIR') or dane.get('cwd') or os.getcwd()
    if dane.get('tool_name') != 'Bash' or not wczytaj_config(root)['wlaczone']:
        sys.exit(0)
    cmd = (dane.get('tool_input') or {}).get('command') or ''
    if 'commit' in punkty_z_komendy(cmd, aliasy_gita(root)):
        dowod_po_commicie(root, cmd)
    tekst = status_po_pushu(root, cmd)
    if tekst:
        print(json.dumps({'hookSpecificOutput': {'hookEventName': 'PostToolUse',
                                                 'additionalContext': tekst}}, ensure_ascii=False))
    sys.exit(0)


def tryb_punkt(punkt, od):
    root = os.getcwd()
    if not wczytaj_config(root)['wlaczone']:
        print('Kontrole wyłączone (## Kontrole → Włączone: nie).')
        return 0
    b, o, e, _ = przebieg(root, punkt, od=od)
    if b:
        print(f'BLOKUJE ({len(b)}):\n' + opisz(b, 200))
    if o:
        print(f'OSTRZEGA ({len(o)}):\n' + opisz(o, 200))
    for x in e:
        print(f'BŁĄD KONTROLI: {x}')
    if not (b or o or e):
        print(f'Kontrole ({punkt}): czysto.')
    return 1 if (b or e) else 0


def main(argv):
    if '--pretooluse' in argv:
        tryb_pretooluse()
        return 0
    if '--posttooluse' in argv:
        tryb_posttooluse()
        return 0
    if '--waliduj' in argv:
        root = argv[argv.index('--waliduj') + 1] if len(argv) > argv.index('--waliduj') + 1 else os.getcwd()
        print('\n'.join(waliduj(root)))
        return 0
    if '--punkt' in argv:
        i = argv.index('--punkt')
        punkt = argv[i + 1] if i + 1 < len(argv) else ''
        if punkt not in PUNKTY:
            sys.stderr.write(f'--punkt: jeden z {", ".join(PUNKTY)}\n')
            return 2
        od = argv[argv.index('--od') + 1] if '--od' in argv and argv.index('--od') + 1 < len(argv) else None
        return tryb_punkt(punkt, od)
    sys.stderr.write(__doc__)
    return 2


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
