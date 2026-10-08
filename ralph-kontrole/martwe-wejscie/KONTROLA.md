---
rodzaj: komenda
uruchom: python3 run.py
sprawdz: python3 run.py --sprawdz
kiedy: faza, wydanie, ci
zakres: całość
przy_bledzie: ostrzega
---

# Kontrola: martwe wejście

Najdroższa klasa defektów z pierwszego retro: mechanizm **zbudowany, przetestowany
i do niczego nie podpięty** — toolbar renderowany tylko w swoim teście, cztery ekrany bez wpisu
w nawigacji, „faza nie działała end-to-end mimo 37 zielonych testów". Reguła `AC: wejście`
(sekcja 0.3 instrukcji) każe nazwać, co uruchamia nowy mechanizm — ta kontrola jest jej
mechanicznym strażnikiem.

**Co łapie.** Symbol **nowy od punktu odniesienia**, do którego poza testami nic się nie odwołuje —
ani inny plik kodu, ani inna linia własnego pliku niż linia definicji. Wywołanie we własnym module
jest wejściem: pierwszy pomiar na repozytorium Ralpha bez tej zasady dał 146 „martwych" funkcji,
z czego każda była helperem wołanym we własnym pliku; z nią — jedno trafne znalezisko. Punkt odniesienia: ostatni tag `ralph/faza-*` (przy `faza`),
ostatni tag `vX.Y.Z` (przy `wydanie`), zmienna `RALPH_OD` (CI: `RALPH_OD=origin/main`), a bez
tagu — pierwszy commit repozytorium. Nowe definicje = linie dodane w `git diff <od>...HEAD`.
Istniejący martwy kod jest celowo pomijany: pełna lista przy każdej fazie byłaby szumem.

| Reguła | Co | Waga |
|---|---|---|
| `symbol-bez-wejscia` | nowa funkcja / klasa / eksport bez odwołania w innym pliku kodu | ostrzega |
| `trasa-bez-klienta` | nowa trasa HTTP (`@app.get`, `router.post`, `@Get()`), której ścieżki nie woła żaden plik frontu / klienta / dokumentacji API | ostrzega |
| `komponent-bez-rodzica` | nowy komponent React (`export function Nazwa` w `.tsx`/`.jsx`) nie renderowany w żadnym innym pliku | ostrzega |
| `martwy-symbol-narzedzie` | symbol nowy w diffie, który `vulture` / `knip` / `ts-prune` zgłasza jako nieużywany, a nasz grep znalazł jego nazwę gdzie indziej (np. w stringu) | ostrzega |

**Nigdy nie blokuje.** Martwy kod nie psuje działania, a precyzja tej kontroli nie jest jeszcze
zmierzona — telemetria (odsetek ostrzeżeń zamkniętych wyjątkiem vs poprawką) powie, czy działa.
Powyżej 30 znalezisk reszta idzie jako jedno znalezisko-licznik.

**Języki i definicje.** Python: `def` / `async def` / `class` na poziomie modułu (bez wcięcia;
metody pomijane). JS/TS: `export function|const|let|class`, `export default function`,
`export { … }` (bez `from` — re-eksport to nie definicja). Go: `func` z dużej litery (eksportowany),
bez metod. Inne języki — zero znalezisk.

**Trasy HTTP** sprawdzane są **tylko, gdy repo ma front**: jakikolwiek plik `.tsx/.jsx/.vue/.svelte/.html`,
`openapi*`/`swagger*`, albo `.ts/.js` w katalogu `frontend/ client/ web/ ui/ static/ public/`. Sam
backend oznacza, że klient jest w innym repozytorium — tras nie oceniamy. Odwołanie do trasy =
jej ścieżka (prefiks `/api`, `/api/vN` opcjonalny; parametry `:id`, `{id}`, `<id>`, `[id]`
dopasowane do dowolnego segmentu) w pliku frontu / klienta / `.md` / `.yaml` / `.json` poza
definicją i testami. Prefiks `APIRouter(prefix=…)`, `Blueprint(url_prefix=…)`, `@Controller(…)` jest
doliczany, gdy stoi w tym samym pliku; trasa, której ścieżka po normalizacji to `/`, nie jest oceniana.

**Czego nie robi / co pomija (punkty wejścia rejestrowane przez konwencję).**
- Python: nazwy z `_`, `main`, `test_*`, klasy `Test*` (unittest odkrywa je po nazwie), nazwy w `__all__`, pliki `migrations/`, `alembic/`,
  `commands/`, `management/`, `manage.py`, `wsgi.py`, `asgi.py`, `conftest.py`, `setup.py`, `settings.py`,
  `urls.py`, `admin.py`, `apps.py`, `signals.py`; `tasks.py`/`celery.py` z `@shared_task`/`celery`;
  `cli.py` z `@click`/`@app.command`/typer; funkcje pod dekoratorami rejestrującymi (`@shared_task`,
  `@click.*`, `@receiver`, `@pytest.fixture`, `@app.on_event`, `@app.exception_handler`, `@validator`…);
  symbol przekazany w tym samym pliku do `register(`/`setup(`/`add_command(`/`include_router(`.
- JS/TS: `index.*` i `main.*` (re-eksporty, bootstrap), `*.d.ts`, `*.config.*`, `*.stories.*`,
  `pages/`, `app/`, `routes/` **tylko** w projektach z routingiem plikowym (Next, SvelteKit, Remix, Nuxt,
  Astro, TanStack/React Router — po pliku konfiguracji albo zależności w `package.json`); hook `use*`
  to zwykły symbol (nie jest pomijany).
- Go: `main`, `init`, `Test*`, `Benchmark*`, `Example*`, `Fuzz*`, metody.
- Pliki testów (`tests/`, `test_*`, `*.test.*`, `*.spec.*`, `_test.*`) nie dostarczają ani definicji,
  ani odwołań; `*.stories.*` tak samo. Katalogi `node_modules/`, `vendor/`, `dist/`, `build/`, `.venv/`,
  pliki > 1 MB, lockfile'e, zminifikowane i binarne — poza korpusem.
- **Uproszczenie:** komentarze nie są wycinane — nazwa wspomniana w komentarzu innego pliku liczy się
  jako odwołanie (kontrola woli przemilczeć niż fałszywie alarmować). Nie śledzi przepływu: `getattr`,
  rejestry stringowe (`"moduł:funkcja"` w konfiguracji) są odwołaniem, bo nazwa pada jako słowo.
  Trasa wołana bez wiodącego `/` (`api.get('items')`) albo składana z kawałków nie jest widziana.

**Narzędzia (dodatkowy sygnał, nic nie jest instalowane).** `vulture` z PATH na nowych plikach
Pythona (`--min-confidence 80`); `knip` / `ts-prune` przez `npx --no-install` tylko gdy stoją
w `devDependencies` projektu. Wynik narzędzia dotyczący symbolu nowego w diffie: potwierdzenie
w opisie znaleziska albo osobne znalezisko `martwy-symbol-narzedzie`. Limit 60 s na narzędzie;
timeout = narzędzie pominięte. Brak narzędzi to normalny tryb (wbudowany, bez uwagi walidatora);
`--sprawdz` ostrzega tylko, gdy projekt deklaruje narzędzie, którego nie ma pod ręką.

Fałszywy alarm (plugin ładowany dynamicznie, publiczne API biblioteki) → wyjątek po odcisku
w `ralph/KONTROLE_WYJATKI.md` (odcisk = plik definicji + nazwa, przeżywa przesunięcie linii) —
wpisuje człowiek.
