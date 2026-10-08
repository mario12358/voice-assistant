---
tag: caddy
keywords: caddy, reverse proxy, tls, https, caddyfile, ingress, rate limit, brzeg
---

# Wytyczne: Caddy

## Rola

- Caddy jest jedynym wejściem HTTP/HTTPS do systemu: TLS, reverse proxy, podstawowa konfiguracja brzegu
- Aplikacja i PostgreSQL nigdy nie są wystawione bezpośrednio do internetu

## Konfiguracja

- `Caddyfile` wersjonowany w repozytorium, montowany do kontenera (alternatywnie generowany z konfiguracji kontenerów — wybór odnotować w PROJECT_CONTEXT)
- `reverse_proxy` do aplikacji po NAZWIE usługi w prywatnej sieci kontenerowej (`app:8000`), nie po IP
- Automatyczny TLS (Let's Encrypt) — nie wyłączaj bez powodu; wolumen na `caddy_data`, żeby restart nie wybijał limitów wydawania certyfikatów
- Admin API Caddy niedostępne z zewnątrz

## Nagłówki i higiena brzegu

- Security headers na brzegu: `Strict-Transport-Security`, `X-Content-Type-Options`, ochrona przed framingiem (CSP `frame-ancestors`)
- Jawne limity rozmiaru body dla endpointów przyjmujących dane
- Jawne timeouty proxy

## Rate limiting

- Ograniczanie liczby żądań NIE jest standardowym elementem podstawowego Caddy — wymaga modułu albo realizacji w warstwie aplikacyjnej
- Wybór warstwy (Caddy z modułem vs middleware w aplikacji) to decyzja przy wykonaniu — odnotuj ją i obejmij ochroną kosztowne operacje

## Logi

- Ustrukturyzowane logi dostępu (JSON) — wejście do diagnostyki i telemetrii
- Nie loguj sekretów/tokenów z nagłówków

## Czego unikać

- Wystawiania portów aplikacji/bazy "na chwilę" obok Caddy
- Konfiguracji brzegu robionej ręcznie na serwerze, poza repozytorium
- Terminowania TLS w aplikacji zamiast w Caddy
