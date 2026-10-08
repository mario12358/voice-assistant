---
rodzaj: komenda
uruchom: python3 run.py
sprawdz: python3 run.py --sprawdz
kiedy: commit, faza, ci
zakres: zmienione
przy_bledzie: blokuje
---

# Kontrola: sast

„Uważaj na OWASP" w wytycznych działa na błędy niewiedzy, nie na błędy z rozpędu: model zna
parametryzowane zapytania i mimo to wkleja f-string do `execute()`, bo w tej jednej linii tak było
szybciej. Ta kontrola czyta zmienione pliki **przed** commitem i szuka kształtów kodu, które są
wstrzyknięciem albo do niego prowadzą.

**Narzędzie.** Gdy w PATH jest `semgrep`, kontrola uruchamia go z pakietami `p/owasp-top-ten`,
`p/security-audit` i pakietem językowym dla rozszerzeń obecnych w zakresie (`p/python`,
`p/javascript`, `p/typescript`, `p/golang`). Pakiety `p/…` semgrep pobiera z sieci przy pierwszym
użyciu; brak sieci, błąd albo przekroczony czas (15 s przy commicie, 240 s przy fazie / CI) to **nie**
blokada — kontrola przechodzi na wzorce wbudowane i dokłada ostrzeżenie `nie-sprawdzono`, żeby było
widać, że przebieg był płytszy. Bez semgrepa działa tylko tryb wbudowany (walidator przy starcie mówi,
który tryb jest aktywny). Wzorce wbudowane działają też **obok** semgrepa — dokładają linie, których
on nie zgłosił (ta sama linia zgłoszona przez semgrepa nie jest liczona drugi raz).
`RALPH_SEMGREP=brak` w środowisku wyłącza semgrepa (testy harnessu).

## Co blokuje, a co ostrzega

Wagę ustala reguła, nie narzędzie: blokują wstrzyknięcia (SQL, komenda, kod), deserializacja
i ścieżka z żądania — klasy, w których jedno trafienie to gotowy exploit. Reszta ostrzega, bo
wymaga oceny kontekstu. Config może złagodzić cały moduł do `ostrzega`, nigdy zaostrzyć.

| Reguła | Waga | Co łapie (w tej samej linii) |
|---|---|---|
| `sql-konkatenacja` | **blokuje** | `execute(` / `executemany(` / `raw(` / `text(` / `query(` z argumentem **sklejanym**: f-string z `{}`, `"…" +`, `% (`, `.format(`; w JS template literal z `${}`. Zmienna sama (`execute(query)`) przechodzi — może być przygotowanym zapytaniem. Parametry (`execute("… %s", (x,))`) przechodzą |
| `komenda-powloki` | **blokuje** | `subprocess.*(…, shell=True)` i `os.system` / `os.popen` z czymkolwiek poza literałem (zmienna, f-string, `+`, `.format`); JS `exec(` / `execSync(` z `${}` albo `+` |
| `eval-exec` | **blokuje** | Python `eval(` / `exec(` z nie-literałem (`eval("1+1")` przechodzi, `model.eval()`, `self.exec(…)` i definicja metody `def exec(` nie są liczone); JS `eval(` z nie-literałem, `new Function(`, `setTimeout("…")`, `vm.runIn*Context(` |
| `deserializacja` | **blokuje** | `pickle.load(s)`, `cPickle`, `marshal.loads`, `shelve.open`, `jsonpickle.decode`, `yaml.load(` bez `SafeLoader` / `BaseLoader` (`safe_load` przechodzi), JS `unserialize(`, Java `ObjectInputStream` |
| `path-traversal` | **blokuje** | `open(` / `Path(` / `send_file(` / `sendFile(` / `FileResponse(` / `os.path.join(` / `readFile(` z `request.` / `req.params` / `req.query` / `req.body` / `params[` / `args.get(` w argumencie — chyba że w tej linii jest `secure_filename(` / `basename(` / `safe_join(` / `realpath(` / `path.resolve(` ze `startsWith(` |
| `xss-innerhtml` | ostrzega | `innerHTML =` / `outerHTML =` (poza pustym literałem), `insertAdjacentHTML`, `dangerouslySetInnerHTML`, `document.write`, `v-html`, `{@html`, Jinja `\|safe`, `Markup(`, `mark_safe(`, Angular `bypassSecurityTrust*` |
| `cors-dowolny-z-credentials` | ostrzega | `credentials: true` / `supports_credentials=True` / `allow_credentials=True` i `origin: "*"` (albo `true`) w tej samej linii lub ±3 linie. Samo `*` bez credentials zgłasza `oslabienia` |
| `slaby-hash` | ostrzega | `hashlib.md5/sha1`, `createHash('md5'/'sha1')`, `MessageDigest MD5/SHA-1`, PHP `md5(` — tylko w linii ze słowem `password` / `passwd` / `haslo` / `token` / `secret`. `md5(file_bytes)` do sum kontrolnych przechodzi |
| `losowosc-niekryptograficzna` | ostrzega | `random.random/randint/choice/…`, `Math.random(`, `rand(` — w linii ze słowem `token` / `secret` / `session` / `otp` / `nonce` / `password` / `reset` / `csrf` / `salt` |
| `redirect-otwarty` | ostrzega | `redirect(` / `res.redirect(` / `RedirectResponse(` z `request.args` / `req.query` / `args.get(` / `next` / `return_to` / `redirect_url` — bez `url_has_allowed_host` / `is_safe_url` / `startswith("/")` / `url_for(` |
| `debug-stacktrace-w-odpowiedzi` | ostrzega | `traceback.format_exc()` / `err.stack` w linii z `return` / `res.send` / `jsonify` / `JSONResponse` / `render`. Ten sam stack w `logger.error` przechodzi |
| `jwt-bez-weryfikacji` | ostrzega | `verify_signature: False`, `algorithms: ['none']`, `verify=False` obok `jwt` / `decode(`, Python `jwt.decode(` bez `algorithms=` w całym wywołaniu (argumenty w kolejnych liniach się liczą), JS `jwt.decode(` (nie weryfikuje podpisu — `jwt.verify` tak) |
| `xxe` | ostrzega | `etree.parse/fromstring/XMLParser`, `minidom.parse`, `xml.sax`, `pulldom`, `resolve_entities=True`, Java `DocumentBuilderFactory` / `SAXParserFactory`. Plik, który gdziekolwiek importuje `defusedxml`, jest z tej reguły zwolniony |
| `ssrf` | ostrzega | `requests.get/post/…`, `httpx.*`, `fetch(`, `axios`, `urlopen(`, `got(` z `request.` / `req.query` / `req.body` / `params[` / `args.get(` w argumencie |
| `tmp-niebezpieczny` | ostrzega | `tempfile.mktemp(`, `open("/tmp/…")`, `os.tmpnam` |
| `logowanie-sekretu` | ostrzega | `print(` / `logger.*(` / `console.log(` / `log.Printf(` z identyfikatorem, którego **ostatnie słowo** to `password` / `secret` / `api_key` / `token` / `authorization` / `private_key` / `credentials` (także `user.password`, `${accessToken}`, `{token}` w f-stringu). Komunikat `"password changed"` to literał — przechodzi; `token_count`, `tokens` — nie kończą się słowem — przechodzą; `len(password)` / `mask(` / `***` przechodzą |
| `nie-sprawdzono` | ostrzega | Semgrep był dostępny, ale nie dał wyniku (sieć, czas, kod błędu) — przebieg zrobiony tylko wzorcami wbudowanymi |

W trybie semgrepa reguła to ostatni segment `check_id`; **blokuje** tylko znalezisko o wadze ERROR,
którego `check_id` zawiera `injection` / `sqli` / `sql-injection` / `command` / `subprocess` / `eval` /
`exec` / `deserial` / `pickle` / `yaml-load` / `path-traversal` / `tainted-path` / `ssrf` — resztę
(także ERROR) sprowadza do ostrzeżenia. W opisie jest komunikat semgrepa i fragment linii ≤ 80 znaków,
nigdy cały blok `extra.lines`.

## Co pomija

- **Pliki testów** w całości (`tests/`, `__tests__/`, `spec/`, `e2e/`, `test_*`, `*_test.*`, `*.spec.*`,
  `*.test.*`) — test legalnie robi `eval`, sklejony SQL i `pickle`.
- `migrations/`, `alembic/`, `seeds/`, `*.sql` — tylko regułę `sql-konkatenacja` i reguły semgrepa
  o SQL (`check_id` z „sql", np. `avoid-sqlalchemy-text`): migracje budują SQL ze stałych (nazwy
  tabel w f-stringach), parametryzować tam nie ma czego.
- `logowanie-sekretu`: wartość przechodząca przez funkcję maskującą (`len(`, `mask`, `redact`,
  `marker` — np. `key_marker(api_key)`, `fingerprint`, `[:4]`) nie jest sekretem w logu.
- Pliki spoza listy rozszerzeń kodu (`.py .js .jsx .ts .tsx .mjs .cjs .vue .svelte .java .kt .scala
  .go .rb .php .cs` + szablony `.html .jinja .j2 .ejs .hbs .njk`), binaria, linie > 4000 znaków,
  linie komentarza i linie `import` / `require(`.
- Linia z komentarzem `# nosec`, `// nosec`, `# nosemgrep`, `// nosemgrep`, `# noqa: S…` —
  świadoma decyzja autora. **Ma być rzadka**: `nosec` znika w kodzie i nikt go później nie policzy,
  a `ralph: osłabienie <id> — <powód>` jest rejestrem, który przypomina o sobie przy fazie i blokuje
  wydanie. Osłabienie świadome oznaczaj znacznikiem, `nosec` zostaw na fałszywe trafienie w linii,
  której nie da się przepisać.
- `verify=False` dla TLS, `DEBUG = True`, `csrf_exempt`, CORS `*` bez credentials — to robi `oslabienia`,
  tu celowo nie zdublowane. ReDoS — pominięty (bez analizy automatu to tylko szum).

## Czego nie robi

Wzorce wbudowane patrzą na **jedną linię** (CORS: ±3). `url = request.args["u"]` w linii wyżej
i `requests.get(url)` niżej przejdzie — to zadanie dla semgrepa z analizą przepływu, nie dla regexa.
Nie śledzi typów: `execute(query)` przechodzi, bo nie wiadomo, skąd `query`. Dlatego pierwsze
uruchomienie na projekcie, który nie miał tej kontroli, warto zrobić ręcznie
(`python3 ralph-kontrole/ralph-kontrole.py --punkt faza`) i skalibrować wyjątki, zanim trafi do commitów.

## Fałszywe alarmy

Jedno znalezisko na regułę na plik, z licznikiem `(+N dalej w pliku)`. Odcisk liczony jest ze
**wszystkich** trafionych linii tej reguły w pliku (nie z pierwszej): wyjątek w
`ralph/KONTROLE_WYJATKI.md` zdejmuje dokładnie te linie, które człowiek widział. Przeżywa
przesunięcie linii; nowa trafiona linia w tym samym pliku zmienia odcisk i blokuje od nowa — to
celowe. Wyjątek wpisuje człowiek; Ralph zgłasza fałszywe trafienie jako RALPH BLOCKED z odciskiem.
