#!/usr/bin/env python3
"""Kontrola `oslabienia` — lista plików na stdin, znaleziska JSON w liniach na stdout."""
import hashlib
import json
import os
import re
import sys

ZNACZNIK = re.compile(r'ralph:\s*os[łl]abienie\b(.*)$', re.I)
ZNACZNIK_PELNY = re.compile(r'^\s+([A-Za-z0-9][\w.-]*)\s*[—–:-]+\s*(\S.{3,})$')
MUTANT = re.compile(r'\bMUTANT\b')

# (reguła, wzorzec, opis) — typowe osłabienia; bez znacznika tylko ostrzegają
WZORCE = [
    ('tls-wylaczony', r'verify\s*=\s*False|rejectUnauthorized\s*:\s*false|NODE_TLS_REJECT_UNAUTHORIZED'
                      r'|InsecureSkipVerify\s*:\s*true|ssl\.CERT_NONE|check_hostname\s*=\s*False'
                      r'|--insecure\b|\bcurl\b[^\n]*\s-k\b',
     'weryfikacja TLS wyłączona'),
    ('debug-wlaczony', r'^\s*DEBUG\s*=\s*True\b|\.run\([^)]*debug\s*=\s*True|app\.debug\s*=\s*True',
     'tryb debug włączony na stałe'),
    ('csrf-wylaczony', r'@csrf_exempt|csrf\s*[:=]\s*(False|false)|disable_csrf|csrf_protect\s*=\s*False',
     'ochrona CSRF wyłączona'),
    ('obejscie-zabezpieczenia', r'(?i)\b(skip|bypass|disable|disabled|no)[_-]?(auth|authn|authz|login|mfa|2fa|totp|'
                                r'otp|captcha|csrf|ratelimit|rate_limit|permission|permissions|verification)\b',
     'obejście zabezpieczenia w identyfikatorze'),
    ('token-obejscia', r'(?i)\b\w*(dev|debug|test|local)[_-]?bypass\w*|\bbypass[_-]?(token|key|secret|user|login)\w*',
     'token / tryb obejścia (zwykle zależny od środowiska — błędna zmienna na produkcji otwiera drzwi)'),
    ('cors-dowolny', r'''Access-Control-Allow-Origin["']?\s*[:,=]\s*["']\*|origin\s*:\s*["']\*["']''',
     'CORS dla dowolnego pochodzenia'),
    ('sekret-w-odpowiedzi', r'(?i)(totp|otp|mfa)[\w.]*\s*(code|kod)\b[^\n]*(render|template|response|json|print|log)',
     'kod drugiego składnika trafia do odpowiedzi / logu'),
]
SKOMPILOWANE = [(r, re.compile(w), o) for r, w, o in WZORCE]
PLIK_TESTU = re.compile(r'(^|/)(tests?|__tests__|spec|e2e)/|(^|/)test_[^/]*$|[._-](test|spec)\.[^/]+$|_test\.[^/]+$')
KOD = re.compile(r'\.(py|js|jsx|ts|tsx|mjs|cjs|java|kt|go|rb|php|rs|cs|swift|scala|sh|sql|ya?ml|toml|json|ini|cfg|conf|env|html|vue|svelte)$|(^|/)(Dockerfile|Caddyfile|Makefile)$')


def odcisk(t):
    return hashlib.sha256(t.encode('utf-8', 'replace')).hexdigest()[:16]


def wypisz(**z):
    print(json.dumps(z, ensure_ascii=False))


def main():
    punkt = os.environ.get('RALPH_PUNKT', 'commit')
    otwarte = {}     # id osłabienia → [(plik, linia, powód)] — jedno osłabienie, wiele miejsc
    root = os.environ.get('RALPH_ROOT') or os.getcwd()
    pliki = [l.strip() for l in sys.stdin.read().splitlines() if l.strip()]
    for p in pliki:
        if not KOD.search(p):
            continue
        try:
            with open(os.path.join(root, p), encoding='utf-8', errors='replace') as f:
                wiersze = f.read().splitlines()
        except OSError:
            continue
        test = bool(PLIK_TESTU.search(p))
        trafienia = {}
        for nr, linia in enumerate(wiersze, 1):
            if len(linia) > 4000:
                continue
            if MUTANT.search(linia):
                wypisz(plik=p, linia=nr, regula='mutant', waga='blokuje',
                       opis='MUTANT w kodzie — test mutacyjny nie został cofnięty',
                       odcisk_tresci=odcisk(linia.strip()))
            m = ZNACZNIK.search(linia)
            if m:
                reszta = m.group(1).rstrip(' */->#').rstrip()
                pm = ZNACZNIK_PELNY.match(reszta)
                if not pm:
                    wypisz(plik=p, linia=nr, regula='znacznik-niepelny', waga='blokuje',
                           opis='znacznik osłabienia bez id albo bez powodu — '
                                'format: ralph: osłabienie <id> — <powód i warunek usunięcia>',
                           odcisk_tresci=odcisk(linia.strip()))
                elif punkt in ('wydanie', 'faza', 'ci'):
                    otwarte.setdefault(pm.group(1), []).append((p, nr, pm.group(2)))
                continue
            if test:
                continue
            for regula, wz, opis in SKOMPILOWANE:
                if wz.search(linia):
                    poprzednia = wiersze[nr - 2] if nr > 1 else ''
                    if ZNACZNIK.search(poprzednia):
                        break        # oznaczone w linii wyżej — rejestr już je ma
                    trafienia.setdefault(regula, []).append((nr, linia, opis))
                    break
        # Wzorzec raz na plik: mechanizm wspominany w 15 liniach to jedno osłabienie,
        # nie piętnaście — reszta idzie jako licznik
        for regula, lista in trafienia.items():
            nr, linia, opis = lista[0]
            dalej = f' (+{len(lista) - 1} dalej w pliku)' if len(lista) > 1 else ''
            wypisz(plik=p, linia=nr, regula=regula, waga='ostrzega',
                   opis=f'{opis} bez znacznika{dalej} — usuń albo oznacz '
                        f'„ralph: osłabienie <id> — <powód>"',
                   odcisk_tresci=regula)
    # Jedno osłabienie = jedno znalezisko, nawet gdy znacznik stoi w pięciu plikach (moduł,
    # montowanie, test, listy wyjątków strażników — tak oznaczył je Ralph na Specky). Odcisk
    # to samo id, bez pliku: świadome wydanie z osłabieniem to jeden wyjątek, nie pięć.
    for oid, miejsca in sorted(otwarte.items()):
        p, nr, powod = miejsca[0]
        gdzie = ', '.join(f'{m[0]}:{m[1]}' for m in miejsca[1:4]) + (' …' if len(miejsca) > 4 else '')
        dalej = f'; też w: {gdzie}' if len(miejsca) > 1 else ''
        if punkt == 'wydanie':
            wypisz(plik=p, linia=nr, regula='oslabienie-otwarte', waga='blokuje', odcisk_pliku=False,
                   opis=f'otwarte osłabienie „{oid}" ({powod[:120]}){dalej} — usuń przed wydaniem '
                        f'(wszystkie znaczniki) albo poproś człowieka o wyjątek',
                   odcisk_tresci=oid)
        else:
            wypisz(plik=p, linia=nr, regula='oslabienie-otwarte', waga='ostrzega', odcisk_pliku=False,
                   opis=f'otwarte osłabienie „{oid}" ({len(miejsca)} miejsc) — zablokuje wydanie',
                   odcisk_tresci=oid)
    return 0


if __name__ == '__main__':
    sys.exit(main())
