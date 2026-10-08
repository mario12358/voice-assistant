---
tag: hono
keywords: hono, api, http, routing, middleware, endpoint, rest, zod, walidacja, kontrakt
---

# Wytyczne: Hono 4

## Struktura

- Jeden moduł domeny = jeden plik routera (`routes/rezerwacje.ts` z własnym `new Hono()`), składane w aplikację przez `app.route("/rezerwacje", rezerwacje)`
- Handler jest cienki: walidacja → wywołanie warstwy domenowej/bazy → mapowanie na odpowiedź. Zero logiki biznesowej w handlerach
- Typowany kontekst: `new Hono<{ Variables: {...} }>()`; dane żądania przez `c.set`/`c.get`, nie przez zmienne globalne

## Walidacja wejścia

- Każde wejście (body, query, param) walidowane NA GRANICY: zod + `@hono/zod-validator`
- Schematy zod współdzielone z frontendem tam, gdzie realnie zapobiega to rozjazdowi kontraktu
- Walidacja aplikacyjna to UX i szybki fail — źródłem integralności pozostają constraints w bazie

## Błędy

- `HTTPException` dla błędów oczekiwanych; centralny `app.onError` mapuje wyjątki na odpowiedzi
- Spójny format błędu w całym API (kod, komunikat, opcjonalnie szczegóły pól)
- Nie zwracaj stack trace ani szczegółów SQL do klienta — szczegóły idą do logów
- 401 vs 403 vs 404 świadomie: brak uprawnień do cudzego zasobu zwykle jako 404 (nie ujawniaj istnienia)

## Middleware

- Kolejność jawna i stała: logger/trace → auth → kontekst tenanta → routery
- Middleware auth ustawia tożsamość w kontekście; do PostgreSQL przekazuje się ją per-transakcja (`SET LOCAL` — patrz wytyczne postgres)

## Testy

- Testy kontraktu API przez `app.request()` — bez wystawiania portu
- Testuj też przypadki negatywne: brak auth, cudzy tenant, niepoprawne wejście

## Czego unikać

- Logiki biznesowej i surowego SQL bezpośrednio w handlerach
- Walidacji "gdzieś w środku" zamiast na granicy
- Autoryzacji rozproszonej w if-ach per endpoint — użyj middleware + granicy w bazie
- Globalnego stanu modułu do przenoszenia danych żądania
