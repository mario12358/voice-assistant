---
rodzaj: regula
uruchom: python3 run.py
kiedy: commit, ci
zakres: zmienione
przy_bledzie: blokuje
---

# Kontrola: higiena

Rzeczy, które nie powinny trafić do repozytorium — najczęstsze mechaniczne potknięcie
modelu przy commicie: `dist/`, `coverage/`, `.pyc`, zrzut bazy na 40 MB, `package.json`
z nową zależnością bez przebudowanego `package-lock.json`. Dziś pilnuje tego tylko
`Ignoruj przy commit` w configu, które działa wtedy, gdy ktoś je uzupełnił. Ta kontrola
działa zawsze — i podpowiada, co dopisać do `.gitignore`.

## Co blokuje, a co ostrzega

| Reguła | Waga | Co łapie |
|---|---|---|
| `katalog-budowania` | blokuje | plik pod `node_modules/`, `dist/`, `build/`, `out/`, `.venv/`, `venv/`, `__pycache__/`, `coverage/`, `htmlcov/`, `.next/`, `.nuxt/`, `.pytest_cache/`, `.mypy_cache/`, `.ruff_cache/`, `target/` (tylko obok `Cargo.toml` / `pom.xml` / `build.gradle`), `*.pyc`, `*.pyo`, `.DS_Store`. Jedno znalezisko na katalog (pierwszy plik + licznik); odcisk = nazwa katalogu, więc jeden wyjątek obejmuje cały katalog |
| `plik-za-duzy` | blokuje | plik > 5 MB (próg: zmienna `RALPH_HIGIENA_MAX_MB`). Artefakt z `artifacts/` przy `Commit artefakty: tak` nie blokuje — powyżej 50 MB przypomina o Git LFS (sekcja 0.5) |
| `lockfile-nieaktualny` | blokuje | manifest z **zmienioną sekcją zależności** w commicie, a lockfile śledzony w repo i nietknięty: `package.json` ↔ `package-lock.json` / `pnpm-lock.yaml` / `yarn.lock` / `bun.lockb`, `pyproject.toml` ↔ `poetry.lock` / `uv.lock` / `pdm.lock`, `Cargo.toml` ↔ `Cargo.lock`, `go.mod` ↔ `go.sum`, `Gemfile` ↔ `Gemfile.lock`, `composer.json` ↔ `composer.lock`, `deno.json` ↔ `deno.lock` (lockfile w tym samym katalogu). Zmiana samej wersji projektu albo skryptów przechodzi; repo bez lockfile'a = projekt go nie używa, cisza |
| `wersja-plywajaca` | ostrzega | zależność **produkcyjna** bez górnej granicy: `*`, `latest`, `x`, pusta, `>=X` bez `<`, `git+…` bez `#ref` (`package.json` → tylko `dependencies`; `requirements*.txt` → linia bez `==` / `~=` / `<` / `===`; `pyproject.toml` → `[project] dependencies` i `[tool.poetry.dependencies]`; `Cargo.toml` → `[dependencies]`). Raz na plik, do 5 nazw w opisie |
| `plik-binarny` | ostrzega | bajt `\0` w pierwszych 8 KB pliku pod `src/`, `app/`, `lib/`, `backend/`, `frontend/`, `pkg/`, `cmd/` — poza obrazami (`.png .jpg .gif .webp .ico .svg`), fontami (`.woff .woff2 .ttf .otf`), `.pdf` w `docs/`, `.wasm`, `bun.lockb` i plikami testów (fikstury) |
| `bundle-zminifikowany` | ostrzega | `.js` / `.css` z linią > 5000 znaków albo nazwą `*.min.*` — poza `vendor/` (także `static/vendor/`, `public/vendor/`) |
| `plik-ide` | ostrzega | `.idea/` (jedno znalezisko na katalog), `.vscode/` poza `extensions.json` / `launch.json` / `tasks.json` / `settings.json` — a te cztery wtedy, gdy zawierają ścieżkę bezwzględną z jednej maszyny (`/Users/`, `/home/`, `C:\`); `*.swp`, `*.swo`, `*~`, `Thumbs.db` |
| `artefakty-wizualne` | ostrzega | plik pod `artifacts/` (pole `Artifacts dir`), gdy `## Artefakty wizualne` → `Commit artefakty` ≠ `tak`. Brak configu albo sekcji = cisza |
| `brak-gitignore` | ostrzega | repo bez `.gitignore` w korzeniu, a do commita wchodzi pierwszy plik kodu. Raz; odcisk bez pliku |

## Skąd bierze listę plików

Runner podaje na stdin pliki zakresu, ale **bez** plików > 1 MB, lockfile'ów i `*.min.*` —
a przy `git add x && git commit` w chwili hooka `x` nie jest jeszcze w indeksie. Dlatego
kontrola dokłada do stdin indeks (`git diff --cached`) i zmiany względem HEAD (przy `ci`:
diff od podstawy gałęzi — `RALPH_OD`, merge-base z `origin/HEAD` / `main`, w ostateczności
`HEAD~1`). Lockfile uznaje za „w commicie", gdy ma jakąkolwiek zmianę względem HEAD —
to jedyny ślad `git add package.json package-lock.json && git commit`, bo runner lockfile
ze stdin wyciął.

## Czego nie robi

- Nie widzi pliku > 5 MB ani `*.min.js`, który jest **nieśledzony i dodawany dopiero
  w komendzie commita** (`git add duzy.sql && git commit`) — nie ma go w indeksie, a runner
  odfiltrował go ze stdin. Plik już w indeksie (osobne `git add`) albo `git add .` jest widoczny.
- Przy `commit` sprawdza także śledzone pliki zmienione, lecz niezaindeksowane (nie odróżnia
  `commit -a` od zwykłego) — zrzut bazy leżący w drzewie roboczym zablokuje commit czegoś
  innego; to i tak sygnał do `.gitignore`.
- Nie czyta lockfile'a — porównuje tylko *zbiór wpisów zależności* manifestu przed i po.
  Lockfile zmieniony ręcznie w złym miejscu przechodzi.
- Nie ocenia wersji pływających w zależnościach deweloperskich, `optional-dependencies`
  ani w `go.mod` (zawsze przypięte).
- Nie sprawdza historii gita — tylko pliki w zakresie punktu.

Fałszywy alarm (katalog `build/` ze źródłami, binarny plik testowy spoza `tests/`, lockfile
przebudowywany tylko w CI) → wyjątek po odcisku w `ralph/KONTROLE_WYJATKI.md`, wpisuje
człowiek. Odcisk `katalog-budowania`, `plik-ide` (`.idea/`) i `artefakty-wizualne` nie
zawiera pliku — jeden wyjątek obejmuje katalog.
