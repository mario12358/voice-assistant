---
rodzaj: komenda
uruchom: python3 run.py
sprawdz: python3 run.py --sprawdz
kiedy: commit, faza, ci
zakres: zmienione
przy_bledzie: blokuje
---

# Kontrola: lint-typy

Linter i typechecker projektu jako strażnik, którego nie trzeba pamiętać. Ralph uruchamia
linter „gdy pamięta" (krok 9 pętli) — ta kontrola robi to przy każdym commicie na zmienionych
plikach, a typechecker przy fazie. Do tego jedna reguła wbudowana, niezależna od narzędzi:
cudzysłów typograficzny w roli ogranicznika stringu — pułapka, która na projekcie
referencyjnym wróciła dziewięć razy w pięć tygodni mimo notatki `📌`.

## Skąd komendy

Z `## Testy` w `ralph/config.md`:

- `Linter/formatter` — np. `ruff check`, `npx eslint .`, `cd backend && ruff check`.
  Placeholder `[np. …]`, `brak` albo puste = bez lintera.
- `Komenda typów` — np. `mypy src`, `pyright`, `npx tsc --noEmit`. `brak` = bez typecheckera.

Bez obu pól działa tylko reguła cudzysłowu (walidator przy starcie mówi o tym jako `uwaga`).

## Co robi w którym punkcie

| Punkt | Cudzysłów | Linter | Typechecker |
|---|---|---|---|
| commit | zmienione pliki kodu | zmienione pliki pasujące do języka narzędzia (limit 15 s) | **pomijany** |
| faza, ci | jak wyżej | pliki z zakresu (limit 240 s) | pełny przebieg (limit 240 s) |

Typechecker nie działa przy commicie celowo: `tsc` i `mypy` nie umieją sensownie „tylko te
pliki", a na dużym projekcie przekraczają budżet commita (20 s na cały moduł).

**Hook nigdy nie zmienia plików.** `--fix` / `--write` są zdejmowane z komendy, formattery
(`black`, `isort`, `prettier`, `cargo fmt`, `ruff format`) dostają `--check`.

## Narzędzia rozpoznawane po nazwie w komendzie

| Narzędzie | Pliki | Format wyjścia |
|---|---|---|
| `ruff`, `flake8`, `pylint`, `black`, `isort` | `.py` | ruff `--output-format json`, pylint `--output-format=json`, flake8 tekst `plik:linia:kol: KOD opis` |
| `eslint`, `biome`, `prettier` | `.js .jsx .ts .tsx .mjs .cjs .vue .svelte` | eslint `-f json`, biome `--reporter=json` |
| `golangci-lint`, `go vet` | `.go` — **bez listy** (pakiety), tylko gdy zmieniono `.go` | golangci `--out-format json` |
| `cargo clippy`, `cargo fmt` | `.rs` — bez listy, tylko gdy zmieniono `.rs` | clippy `--message-format json` |
| `rubocop` | `.rb` | `--format json` |
| `shellcheck` | `.sh .bash` | `-f json` |
| `mypy`, `pyright`, `tsc` (typy) | — | mypy tekst, pyright `--outputjson`, tsc `plik(l,c): error TSxxxx` |

Nieznane narzędzie: przy fazie / CI uruchamiane tak, jak stoi, i czytane parserem tekstowym
(`plik:linia…`); przy commicie pominięte z jednym ostrzeżeniem `linter-nieznany` — nie wiadomo,
które pliki mu podać. Komenda złożona (potok, `;`, `&&` poza prefiksem `cd X &&`) jest
traktowana tak samo. Prefiks `cd X &&` działa: ścieżki plików są przeliczane względem `X`.

**Kilka komend w jednym polu** — projekt z backendem i frontendem wpisuje każdą osobno, ze swoim
`cd`: `` `cd backend && ruff check app tests` · `cd frontend && npx eslint src` `` (komendy
w backtickach albo rozdzielone ` · ` / ` oraz ` / ` ; `). Każda dostaje tylko pliki ze swojego
katalogu. Ścieżki wpisane w komendę za narzędziem (`app tests`, `src`) są przy commicie
zastępowane listą zmienionych plików — inaczej linter sprawdzałby cały katalog i blokował
commit za błędy w plikach, których commit nie dotyka. To samo dla `Komenda typów`.

Gdy po filtrze języka nie ma żadnego pliku (commit samych `.md`), linter nie startuje — cisza.

## Reguły i wagi

| Reguła | Waga | Co |
|---|---|---|
| `cudzyslow-typograficzny` | **blokuje** | `„ ” “ ‚ ’ ‹ › « »` jako ogranicznik: `x = „tekst”`, `foo(“a”)`, `return „b”` albo `"tekst”` (otwarty ASCII, domknięty typograficznie) |
| `linter/<kod>` | wg kodu | ruff/flake8: `E9*`, `F*`, `B*`, błąd składni → blokuje; `E`, `W`, `I`, `D`, `N`, `UP`… → ostrzega. eslint `severity: 2` / fatal, pylint `error`/`fatal`, biome `error`, shellcheck `error`, rubocop `error`/`fatal`, clippy `error` → blokuje; reszta ostrzega |
| `linter/format` | ostrzega | plik do przeformatowania (black / isort / prettier / cargo fmt) |
| `typy/<kod>` | **blokuje** | mypy `error … [kod]`, tsc `error TSxxxx`, pyright `error`; pyright `warning` ostrzega, `note` pomijane |
| `linter-nieznany` | ostrzega | narzędzie nierozpoznane — przy commicie pominięte |
| `linter-niedostepny`, `typy-niedostepne` | ostrzega | komenda wskazuje na coś, czego nie ma (kod 127 / brak pliku) — pole jest opcjonalne, więc nie kod 3 |
| `linter-zglosil`, `typy-zglosily` | ostrzega | kod wyjścia ≠ 0, wynik nieparsowalny — pierwsze 3 linie wyjścia (≤ 300 znaków) |
| `linter-limit-czasu`, `typy-limit-czasu` | ostrzega | przekroczony limit — operacja przeszła bez narzędzia |

Reguły stylu (`E501`, `W…`, `format`) nie blokują nawet wtedy, gdy narzędzie samo
klasyfikuje je jako błąd — blokuje to, co jest błędem składni, niezdefiniowaną nazwą albo
błędem typu. Odcisk znaleziska lintera to treść linii (przeżywa przesunięcie), znaleziska
„o narzędziu" (`linter-*`, `typy-*`) mają `odcisk_pliku: false`.

## Reguła cudzysłowu — co przechodzi

Prosty skaner stanu per linia (w stringu / poza / w komentarzu, z komentarzem blokowym,
docstringiem `"""` i template literalem przenoszonymi między liniami). Cudzysłów typograficzny
**wewnątrz** poprawnego stringu — `print("Powiedział „tak”")`, `` `Cytat „x”` `` — to
najczęstszy przypadek w projektach Ralpha (polskie teksty UI) i nie daje znaleziska. Komentarze
(`#`, `//`, `/* */`, `--`), docstringi, pliki `.md` / `.txt` — pomijane.

Celowe ustępstwa, żeby nie było fałszywych alarmów:

- **shell, YAML** — tylko wariant mieszany (`"tekst”`) i przypisanie `X=„…”`; poza stringiem
  cudzysłów typograficzny jest tam legalnym znakiem (heredoc, wartości w plikach tłumaczeń).
- **JSX/TSX, PHP** — bez reguły „zamykającej" (`…”,`), bo tekst stoi tam poza stringami
  (`<p>Powiedział „tak”, </p>`); `title=„x”` i `{„x”}` dalej łapie.
- Koniec linii po literale nie jest sygnałem — JSX i heredoc rozbijają tekst na linie.

Co pomija: literały regex w JS, string raw w Rust z `#`, lifetime `'a` (reszta linii uznana za
string — co najwyżej przeoczy, nie zgłosi fałszywie).

## Czego nie robi

Nie konfiguruje lintera (brak `eslint.config.js` / `[tool.ruff]` = narzędzie działa z własnymi
domyślnymi regułami albo zgłasza błąd — wtedy `linter-zglosil`). Nie uruchamia testów. Nie
sprawdza plików `.md`. Nie skanuje całego repo przy zakresie `zmienione`.

Fałszywy alarm → wyjątek po odcisku w `ralph/KONTROLE_WYJATKI.md`, wpisuje człowiek.
