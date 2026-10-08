---
tag: htmx
keywords: htmx, ssr, hypermedia, server rendering, formularze, fragment html, hx-get, hx-post
---

# Wytyczne: SSR + HTMX

## Kiedy HTMX, kiedy React

- HTMX: ekran jest przede wszystkim prezentacją danych i formularzem (listy, CRUD, panele)
- React: interfejs z bogatym stanem i interakcją po stronie klienta
- Punktowy vanilla JS: drobna lokalna interaktywność
- Decyzja per ekran — nie ma wymogu jednego modelu renderowania w całej aplikacji

## Model pracy

- Serwer renderuje HTML (`hono/jsx` jako templating); HTMX wymienia fragmenty strony
- Endpoint HTMX zwraca FRAGMENT HTML — nie JSON i nie pełną stronę
- `hx-get`/`hx-post` + `hx-target` + `hx-swap` — jawnie wskazany cel podmiany
- Stan mieszka na serwerze/w bazie — nie buduj modelu stanu w przeglądarce
- Zdarzenia między fragmentami: nagłówek `HX-Trigger` z serwera zamiast ręcznego JS

## Formularze

- Formularz działa też bez JS tam, gdzie to tanie (zwykły POST + redirect) — HTMX jako progressive enhancement
- Błędy walidacji: serwer zwraca fragment formularza z komunikatami (status 422), HTMX podmienia w miejscu
- Po mutacji: odpowiedź zawiera zaktualizowany fragment albo `HX-Redirect`

## Bezpieczeństwo

- Kod w przeglądarce NIGDY nie jest granicą autoryzacji — każdy endpoint HTMX autoryzuje jak zwykły endpoint (auth middleware + role/RLS w bazie)
- CSRF: token w formularzach mutujących; sam nagłówek `HX-Request` NIE jest zabezpieczeniem
- Domyślne escapowanie treści w szablonach; surowy HTML tylko zaufany i sanityzowany

## Czego unikać

- Budowania SPA w HTMX — dużo klienckiego stanu to sygnał, że ekran należy do Reacta
- Endpointów zwracających JSON, który JS składa w HTML — HTML składa serwer
- Duplikowania po stronie klienta danych/stanu, które ma serwer
- Ukrywania elementów UI jako "kontroli dostępu"
