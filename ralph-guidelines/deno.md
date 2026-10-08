---
tag: deno
keywords: deno, deno2, runtime, deno.json, jsr, permissions, tasks, std
---

# Wytyczne: Deno 2

## Konfiguracja projektu

- `deno.json` to centrum projektu: `tasks`, `imports` (import map), `compilerOptions`, `fmt`, `lint`
- NIE twórz `tsconfig.json` ani `package.json` — opcje TypeScript w `compilerOptions` w `deno.json`
- Wersje zależności przypięte w import map, nie rozproszone po plikach
- `deno.lock` commitowany do repo

## Zależności

- Preferuj `jsr:` (JSR) i bibliotekę standardową `@std/*`; `npm:` tylko gdy brak odpowiednika
- Import przez alias z import map (`"hono": "jsr:@hono/hono@^4"`), nie pełne specyfikatory w kodzie
- Nie dodawaj zależności na coś, co jest w `Deno.*` lub `@std/*` (fs, path, crypto, dotenv, testing, assert)

## Uprawnienia (sandbox)

- Uruchamiaj z JAWNYMI, minimalnymi flagami: `--allow-net=host:port`, `--allow-env=NAZWA1,NAZWA2`, `--allow-read=ścieżka`
- `-A` / `--allow-all` co najwyżej lokalnie w dev — nigdy w CI ani w obrazie produkcyjnym
- Flagi uprawnień zapisane w tasks w `deno.json` — jedno źródło prawdy, jak uruchamiać

## Zadania (tasks)

- Każda operacja projektu jako task: `deno task dev`, `test`, `check`, `db:migrate`
- CI wywołuje te same taski co developer — zero komend istniejących "tylko w CI"

## Testy

- `Deno.test` + `@std/assert` / `@std/expect`; pliki `*_test.ts` obok kodu lub w `tests/`
- `deno test` z minimalnymi uprawnieniami — test, który nie potrzebuje sieci, nie dostaje sieci
- Sanitizery zasobów zostają włączone (łapią wycieki połączeń/timerów) — naprawiaj wyciek, nie wyłączaj sanitizera

## Jakość

- `deno fmt` i `deno lint` wbudowane — bez Prettiera/ESLinta; `deno check` w CI przed testami
- Formatowanie nie podlega dyskusji — rozstrzyga fmt

## Kontener

- Obraz z oficjalnego `denoland/deno`; `deno cache` zależności jako osobna warstwa przed COPY kodu
- Uruchomienie w kontenerze z tymi samymi jawnymi flagami uprawnień co w tasks

## Czego unikać

- `npm:` gdy istnieje odpowiednik `jsr:`/`@std/*`
- Node-izmów: `process.env` (użyj `Deno.env.get`), `__dirname` (użyj `import.meta`), `require`
- `-A` w CI/produkcji
- Dynamicznych importów z URL budowanych w runtime
