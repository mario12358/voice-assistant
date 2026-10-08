"""Wspólne dla kontroli, które pytają usługi sieciowe: HTTP z limitem czasu, pamięć
podręczna w ~/.cache/ralph/, adresy usług nadpisywalne zmiennymi (testy podstawiają
lokalny serwer). Moduł importuje go przez `sys.path.insert(0, '..')`.
"""
import json
import os
import ssl
import time
import urllib.error
import urllib.request

ADRESY = {
    'npm': ('RALPH_REJESTR_NPM', 'https://registry.npmjs.org'),
    'npm_pobrania': ('RALPH_REJESTR_NPM_POBRANIA', 'https://api.npmjs.org/downloads/point/last-week'),
    'pypi': ('RALPH_REJESTR_PYPI', 'https://pypi.org/pypi'),
    'jsr': ('RALPH_REJESTR_JSR', 'https://api.jsr.io'),
    'osv': ('RALPH_OSV', 'https://api.osv.dev'),
}


def adres(nazwa):
    zmienna, domyslny = ADRESY[nazwa]
    return os.environ.get(zmienna, domyslny).rstrip('/')


def _konteksty_ssl():
    """Python z python.org na macOS nie ma certyfikatów CA — ta sama kolejność co
    w kolektorze telemetrii: domyślny magazyn, certifi, pakiet systemowy."""
    yield ssl.create_default_context()
    try:
        import certifi
        yield ssl.create_default_context(cafile=certifi.where())
    except ImportError:
        pass
    for p in ('/etc/ssl/cert.pem', '/etc/ssl/certs/ca-certificates.crt'):
        if os.path.exists(p):
            yield ssl.create_default_context(cafile=p)


class BrakSieci(Exception):
    pass


def zapytaj(url, dane=None, limit=6):
    """(status, json|None). 404 → (404, None). Sieć/serwer niedostępny → BrakSieci."""
    tresc = json.dumps(dane).encode('utf-8') if dane is not None else None
    naglowki = {'Accept': 'application/json', 'User-Agent': 'ralph-kontrole'}
    if tresc is not None:
        naglowki['Content-Type'] = 'application/json'
    ostatni = None
    for ctx in (_konteksty_ssl() if url.startswith('https') else [None]):
        req = urllib.request.Request(url, data=tresc, headers=naglowki)
        try:
            with urllib.request.urlopen(req, timeout=limit, context=ctx) as r:
                return r.status, json.loads(r.read().decode('utf-8') or 'null')
        except urllib.error.HTTPError as e:
            if e.code in (404, 410):
                return 404, None
            ostatni = e
            if e.code < 500:
                break
        except urllib.error.URLError as e:
            ostatni = e
            if not isinstance(getattr(e, 'reason', None), ssl.SSLError):
                break
        except (OSError, ValueError) as e:
            ostatni = e
            break
    raise BrakSieci(str(ostatni))


class Pamiec:
    """Słownik zapisywany w ~/.cache/ralph/<nazwa>.json; wpis wygasa po `ttl` s."""

    def __init__(self, nazwa):
        katalog = os.environ.get('RALPH_CACHE') or os.path.expanduser('~/.cache/ralph')
        self.sciezka = os.path.join(katalog, nazwa + '.json')
        try:
            with open(self.sciezka, encoding='utf-8') as f:
                self.dane = json.load(f)
        except (OSError, ValueError):
            self.dane = {}
        self.zmieniona = False

    def daj(self, klucz):
        w = self.dane.get(klucz)
        if w and w.get('wygasa', 0) > time.time():
            return w.get('v')
        return None

    def wstaw(self, klucz, wartosc, ttl):
        self.dane[klucz] = {'v': wartosc, 'wygasa': time.time() + ttl}
        self.zmieniona = True

    def zapisz(self):
        if not self.zmieniona:
            return
        teraz = time.time()
        self.dane = {k: v for k, v in self.dane.items() if v.get('wygasa', 0) > teraz}
        try:
            os.makedirs(os.path.dirname(self.sciezka), exist_ok=True)
            tmp = self.sciezka + '.tmp'
            with open(tmp, 'w', encoding='utf-8') as f:
                json.dump(self.dane, f)
            os.replace(tmp, self.sciezka)
        except OSError:
            pass
