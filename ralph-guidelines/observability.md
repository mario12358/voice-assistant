---
tag: observability
keywords: otel, opentelemetry, otlp, logi, metryki, monitoring, alerty, telemetria, tracing, health
---

# Wytyczne: Obserwowalność

## Zasada

- System emituje ustrukturyzowane logi i podstawową telemetrię OD POCZĄTKU — nie "dodamy później"
- Otwarte standardy: OpenTelemetry i OTLP; pełny stos (Grafana/Prometheus/Loki/Tempo/Alloy) opcjonalnie, gdy potrzebny — nie jest warunkiem startu

## Logi

- Ustrukturyzowane (JSON na stdout), jeden wpis = jedno zdarzenie
- Każdy wpis: poziom, czas, nazwa zdarzenia + correlation/trace id żądania — przewlekany przez całą ścieżkę (middleware → domena → baza)
- Poziomy świadomie: ERROR = wymaga reakcji; WARN = degradacja; INFO = zdarzenia biznesowe; DEBUG domyślnie wyłączony
- NIE loguj danych wrażliwych (dane osobowe/medyczne, tokeny, hasła, pełne payloady) — loguj identyfikatory
- Zdarzenia audytowe to NIE logi — idą do dziennika audytu w bazie (wytyczne postgres)

## Metryki

Minimalny zakres:
- host: CPU, RAM, dysk (miejsce!), IO
- aplikacja: liczba żądań, kody odpowiedzi, latencja jako histogram (percentyle, nie średnia)
- PostgreSQL: połączenia, locki, czas zapytań, rozmiar bazy
- czas wykonywania najważniejszych operacji biznesowych

## Alerty

- Minimum bezwzględne: kończące się miejsce na dysku (z wyprzedzeniem), brak odpowiedzi aplikacji, brak połączenia z bazą
- Monitoring dostępności także SPOZA serwera (zewnętrzny healthcheck) — serwer nie może być jedynym świadkiem własnej awarii

## Health endpoints

- `/healthz` (liveness — proces żyje) i `/readyz` (readiness — baza osiągalna)
- Używane przez healthcheck kontenera i monitoring zewnętrzny; bez danych wrażliwych w odpowiedzi

## Czego unikać

- Logów nieustrukturyzowanych (printf-debugging w produkcji)
- Danych wrażliwych w logach, metrykach i atrybutach spanów
- Średniej jako jedynej miary latencji — patrz p50/p95/p99
- Telemetrii przywiązanej do zamkniętego vendora zamiast OTLP
- Alertów na wszystko — alert, na który nikt nie reaguje, uczy ignorowania alertów
