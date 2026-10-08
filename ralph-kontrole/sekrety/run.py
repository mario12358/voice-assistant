#!/usr/bin/env python3
"""Kontrola `sekrety` — lista plików na stdin, znaleziska JSON w liniach na stdout.

Treść sekretu nigdy nie opuszcza tego procesu: na wyjściu jest tylko odcisk.
"""
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile

# (reguła, wzorzec, waga) — formaty o niskim odsetku fałszywych trafień blokują
WZORCE = [
    ('klucz-prywatny', r'-----BEGIN (?:RSA |EC |DSA |OPENSSH |PGP |ENCRYPTED )?PRIVATE KEY(?: BLOCK)?-----', 'blokuje'),
    ('aws-access-key', r'\b(?:AKIA|ASIA)[0-9A-Z]{16}\b', 'blokuje'),
    ('github-token', r'\b(?:gh[pousr]_[A-Za-z0-9]{36,}|github_pat_[A-Za-z0-9_]{50,})\b', 'blokuje'),
    ('gitlab-token', r'\bglpat-[A-Za-z0-9_-]{20,}\b', 'blokuje'),
    ('slack-token', r'\bxox[abposr]-[A-Za-z0-9-]{10,}\b', 'blokuje'),
    ('anthropic-key', r'\bsk-ant-[A-Za-z0-9_-]{20,}\b', 'blokuje'),
    ('openai-key', r'\bsk-(?!ant-)(?:proj-)?[A-Za-z0-9_-]{32,}\b', 'blokuje'),
    ('google-api-key', r'\bAIza[0-9A-Za-z_-]{35}\b', 'blokuje'),
    ('stripe-live-key', r'\b[rs]k_live_[0-9a-zA-Z]{20,}\b', 'blokuje'),
    ('jwt', r'\beyJ[A-Za-z0-9_-]{10,}\.eyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}', 'ostrzega'),
    ('przypisanie-sekretu',
     r'''(?i)\b(?:password|passwd|pwd|secret|api[_-]?key|access[_-]?token|auth[_-]?token|client[_-]?secret|haslo|hasło)\b["']?\s*[:=]\s*["']([^"'\s]{8,})["']''',
     'ostrzega'),
]
SKOMPILOWANE = [(r, re.compile(w), g) for r, w, g in WZORCE]

# Wartości, które wyglądają na zaślepki, nie na sekrety
ZASLEPKA = re.compile(r'(?i)^(?:x+|\*+|changeme|change[_-]?me|example|sample|dummy|placeholder|'
                      r'your[_-].*|<.*>|\$\{.*\}|\{\{.*\}\}|test|testing|fake|secret|password|todo|none|null)$')
# Celowo bez `test.*`: „Test1234Haslo" to klasyczne słabe hasło, nie zaślepka (pierwszy test
# na Specky przeszedł po cichu). Pliki testów są pomijane w całości, więc szumu to nie dokłada.

# Klucz z dokumentacji / fikstury testu: znacznik w treści albo za mało różnych znaków
FALSZYWKA = re.compile(r'(?i)example|test|fake|dummy|sample|placeholder|x{5}|0{5}|1234|abcd')
PLIK_TESTU = re.compile(r'(^|/)(tests?|__tests__|spec|e2e|fixtures?)/|(^|/)test_[^/]*$|[._-](test|spec)\.[^/]+$|_test\.[^/]+$')
BASE64 = re.compile(r'^[A-Za-z0-9+/=]{40,}$')

PLIK_SEKRETU = re.compile(r'(^|/)(\.env(\.[\w-]+)?|id_(rsa|dsa|ecdsa|ed25519)|[^/]*\.(pem|p12|pfx|key|keystore|jks)|'
                          r'credentials\.json|secrets?\.(ya?ml|json|toml))$')
PLIK_PRZYKLADU = re.compile(r'(?i)(\.|_|-)(example|sample|template|dist|defaults?)(\.|$)')


def odcisk(t):
    return hashlib.sha256(t.encode('utf-8', 'replace')).hexdigest()[:16]


def wypisz(**z):
    print(json.dumps(z, ensure_ascii=False))


def czy_binarny(dane):
    return b'\0' in dane[:8192]


def nazwy_plikow(pliki):
    for p in pliki:
        if PLIK_SEKRETU.search(p) and not PLIK_PRZYKLADU.search(os.path.basename(p)):
            wypisz(plik=p, linia=None, regula='plik-sekretu', waga='blokuje',
                   opis='plik o nazwie sekretu (klucz / .env / credentials) — nie commituj, '
                        'dopisz do .gitignore; wzór trzymaj jako *.example',
                   odcisk_tresci=odcisk('plik:' + p))


def wbudowany(root, pliki):
    for p in pliki:
        try:
            with open(os.path.join(root, p), 'rb') as f:
                dane = f.read()
        except OSError:
            continue
        if czy_binarny(dane):
            continue
        test = bool(PLIK_TESTU.search(p))
        wiersze = dane.decode('utf-8', 'replace').splitlines()
        for nr, linia in enumerate(wiersze, 1):
            if len(linia) > 4000:
                continue
            for regula, wz, waga in SKOMPILOWANE:
                m = wz.search(linia)
                if not m:
                    continue
                wartosc = m.group(1) if m.groups() else m.group(0)
                if regula == 'przypisanie-sekretu' and (test or ZASLEPKA.match(wartosc) or
                                                       re.search(r'(?i)env|getenv|process\.|os\.', linia)):
                    continue
                if regula == 'klucz-prywatny':
                    # sam nagłówek (dokumentacja, komunikat) to nie klucz — liczy się treść pod nim
                    nast = wiersze[nr].strip() if nr < len(wiersze) else ''
                    if not BASE64.match(nast) and not re.search(r'KEY-----\s*(\\n)?[A-Za-z0-9+/=]{40,}', linia):
                        continue
                    wartosc = nast or linia
                elif waga == 'blokuje' and (FALSZYWKA.search(wartosc) or len(set(wartosc)) < 12):
                    continue
                wypisz(plik=p, linia=nr, regula=regula, waga=waga,
                       opis=f'{regula}: {wartosc[:4]}… (treść ukryta)',
                       odcisk_tresci=odcisk(wartosc))


def z_gitleaks(root, pliki):
    """Kopiuje pliki do katalogu tymczasowego i skanuje go jednym wywołaniem.
    Zwraca False, gdy gitleaks nie dał się użyć (wtedy tryb wbudowany)."""
    with tempfile.TemporaryDirectory(prefix='ralph-sekrety-') as tmp:
        for p in pliki:
            cel = os.path.join(tmp, p)
            os.makedirs(os.path.dirname(cel), exist_ok=True)
            try:
                shutil.copyfile(os.path.join(root, p), cel)
            except OSError:
                pass
        raport = os.path.join(tmp, '.raport.json')
        # Konfiguracja projektu (reguły, allowlista) i jego .gitleaksignore — skan idzie
        # z katalogu tymczasowego, więc gitleaks sam by ich nie znalazł
        projekt = []
        if os.path.isfile(os.path.join(root, '.gitleaks.toml')):
            projekt += ['--config', os.path.join(root, '.gitleaks.toml')]
        if os.path.isfile(os.path.join(root, '.gitleaksignore')):
            projekt += ['--gitleaks-ignore-path', os.path.join(root, '.gitleaksignore')]
        for args in (['dir', tmp], ['detect', '--no-git', '--source', tmp]):
            try:
                r = subprocess.run(['gitleaks', *args, *projekt, '--report-format', 'json', '--report-path', raport,
                                    '--exit-code', '0', '--no-banner', '--redact'],
                                   capture_output=True, text=True, timeout=240)
            except (OSError, subprocess.TimeoutExpired):
                return False
            if r.returncode == 0 and os.path.exists(raport):
                break
        else:
            return False
        try:
            with open(raport, encoding='utf-8') as f:
                wyniki = json.load(f) or []
        except (OSError, ValueError):
            return False
        for w in wyniki:
            plik = os.path.relpath(w.get('File', ''), tmp)
            # z --redact pole Secret jest zamazane; odcisk liczymy z oryginalnej linii
            nr = w.get('StartLine')
            try:
                with open(os.path.join(root, plik), encoding='utf-8', errors='replace') as f:
                    tresc = f.read().splitlines()[nr - 1] if nr else ''
            except (OSError, IndexError):
                tresc = w.get('Fingerprint', '')
            wypisz(plik=plik, linia=nr, regula='gitleaks:' + w.get('RuleID', '?'), waga='blokuje',
                   opis=f'{w.get("Description") or w.get("RuleID")} (treść ukryta)',
                   odcisk_tresci=odcisk(tresc.strip()))
    return True


def main():
    if '--sprawdz' in sys.argv:
        if shutil.which('gitleaks'):
            r = subprocess.run(['gitleaks', 'version'], capture_output=True, text=True)
            print(f'ok: gitleaks {r.stdout.strip()}')
        else:
            print('uwaga: gitleaks nie jest zainstalowany — tryb wbudowany (kilkanaście reguł zamiast '
                  'kilkuset); brew install gitleaks')
        return 0
    root = os.environ.get('RALPH_ROOT') or os.getcwd()
    pliki = [l.strip() for l in sys.stdin.read().splitlines() if l.strip()]
    if not pliki:
        return 0
    nazwy_plikow(pliki)
    if not (shutil.which('gitleaks') and z_gitleaks(root, pliki)):
        wbudowany(root, pliki)
    return 0


if __name__ == '__main__':
    sys.exit(main())
