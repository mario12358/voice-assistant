---
rodzaj: regula
uruchom: python3 run.py
sprawdz: python3 run.py --sprawdz
kiedy: commit, faza, ci
zakres: zmienione
przy_bledzie: blokuje
---

# Kontrola: intencja

Pozostałe kontrole czytają linię kodu: łapią wstrzyknięcie wpisane z rozpędu, nie łapią
**brakującego** sprawdzenia. Brakująca autoryzacja (OWASP A01) to najdroższa klasa w hubie:
„lekarz A1 przyjmował pacjenta z kolejki A2" i „ekran podpisu otwierał każdy, kto znał adres"
przeszły przez zielone testy. Ta kontrola porównuje kod z **deklaracją intencji** —
`ralph/BEZPIECZENSTWO.md` (mapa powierzchni ataku, format w `MAPA_SZABLON.md` obok) — więc
nowa trasa bez nazwanej roli i trasa, z której zniknął strażnik, nie przechodzą po cichu.

**Okno trasy.** Kontrola nie rozumie frameworka uwierzytelniania i nie musi. Dla każdej trasy
bierze tekst, w którym stoją strażnicy: dekoratory i sygnaturę funkcji z definicją routera
(`APIRouter(dependencies=…)`) w Pythonie, argumenty wywołania przed handlerem i `x.use(prefiks, …)`
w JS, `@UseGuards` / `@Roles` metody i klasy w Nest. Mapa mówi, jaki tekst oznacza daną rolę
(kolumna „Strażnik w kodzie"). Pod-aplikacja montowana w innym pliku (`app.route("/doctor",
createDoctorRoutes(…))`, `trasyZespolow(routes, …)`) dostaje prefiks i strażników z miejsca
montowania — do trzech poziomów, przez `git grep`; bez tego na altaforcie `GET /` występował
10 razy jako jedna trasa, a 83 z 235 tras wyglądało na niechronione. Rozpoznawane: FastAPI, Flask, Starlette-podobne (`@x.get(` /
`@x.route(`), Express, Hono, Fastify, Koa-router (plik musi importować framework — inaczej
`api.get('/x')` to klient axios), Nest. Inne frameworki (Django URLconf, Rails, Spring) —
kontrola milczy.

**„Nowe"** = nieobecne w tym pliku w punkcie odniesienia: przy `commit` w HEAD, przy `faza`
w ostatnim tagu `ralph/faza-*`, w CI — `RALPH_OD`. Mapa czytana z dysku, więc wiersz dopisany
w tym samym commicie co trasa się liczy.

| Reguła | Waga | Co |
|---|---|---|
| `trasa-poza-mapa` | **blokuje** | nowa trasa bez wiersza w `## Trasy`. Naprawa jest tania — dopisać wiersz — i o to chodzi: wiersz wymaga nazwania ról i własności, jak `AC: wejście` wymaga nazwania wejścia. Trasa sprzed zmiany bez wiersza — ostrzega |
| `straznik-niezgodny` | **blokuje** | w zmienionym pliku trasa z rolą ≠ `anonim`, a w jej oknie nie ma żadnego strażnika z kolumny tej roli. Łapie i nową trasę bez strażnika, i strażnika usuniętego z istniejącej |
| `nowa-publiczna` | ostrzega | nowa trasa z rolą `anonim` — bywa legalna (webhook z podpisem, health), ale zawsze warta linii w raporcie |
| `rola-nieustalona` | ostrzega | trasy z rolą `?` w zmienionym pliku (szkielet nie rozpoznał strażnika): publiczna czy brakuje strażnika? |
| `rola-nieznana` / `rola-bez-straznika` | ostrzega | rola z `## Trasy` bez wiersza w `## Role` / bez „Strażnika w kodzie" — trasy tej roli nie są sprawdzane |
| `trasa-zniknela` | ostrzega | wiersz mapy dla trasy, której w kodzie już nie ma |
| `host-poza-mapa` | ostrzega | nowy `https?://<host>` w kodzie (poza testami, komentarzami, `localhost`, `example.*`) bez wiersza `host` — eksfiltracja, SSRF, instrukcja wstrzyknięta przez `changes/`. Subdomena hosta z mapy przechodzi |
| `sekret-poza-mapa` | ostrzega | nowy odczyt zmiennej środowiskowej o nazwie z `KEY` / `SECRET` / `TOKEN` / `PASSWORD` / `DSN`… bez wiersza `env` |
| `uprawnienie-bazy` | ostrzega | w migracjach i `.sql`: `GRANT`, `SECURITY DEFINER`, `BYPASSRLS`, `DISABLE ROW LEVEL SECURITY`, `DROP POLICY`, `ALTER ROLE` bez wiersza `baza`, którego wartość jest fragmentem tej linii |
| `blob-zakodowany` | ostrzega | literał base64/hex dłuższy niż 200 znaków w nowych liniach (poza `data:image/`) — ukryty ładunek albo osadzony klucz |
| `brak-mapy` | ostrzega | zmienione trasy, a mapy nie ma — nic z powyższego nie jest sprawdzane |

Czego **nie** robi: nie ocenia, czy strażnik jest *właściwy* dla zasobu (lekarz A1 vs pacjent
A2) — to granica własności, a ją sprawdza tylko test z dwoma kontami. Od tego jest kontrola
`red-team`.

**Szkielet.** `python3 run.py --szkielet` (ralph-start.sh robi to sam przy starcie, gdy mapy
brak, a projekt ma trasy) zapisuje mapę z całego repozytorium: wszystkie trasy, role-kandydaci
z nazw w oknie (`current_active_user`, `require_org_admin`, `requireAuth`…), `?` tam, gdzie nic
nie rozpoznał, Własność `?` przy ścieżce z parametrem, hosty i sekrety ze środowiska. Heurystyka
działa **tylko** przy szkielecie — sprawdzenie bierze strażników z mapy. `--wypisz` = na stdout.
Fabryka z literałem daje osobną rolę: `requireRole("doctor")` → `requireRole:doctor`, inaczej
role personelu zlewają się w jedną. Token `requireRole("doctor")` pasuje też do
`requireRole("doctor", "nurse")`. Pomiary: Specky_app_v3 — 234 trasy, 16 z `?`; altaforta
(Hono) — 235 tras, 9 z `?`, siedem ról; we wszystkich przypadkach `?` to trasy z natury publiczne
(health, `.well-known`, webhooki, logowanie, pliki statyczne, iCal z tokenem).

**Pomijane:** pliki testów, katalogi budowania i zależności, `ralph-kontrole/`, `.claude/`, linie
komentarzy (przy hostach, sekretach i blobach).
