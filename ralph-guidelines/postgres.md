---
tag: postgres
keywords: postgresql, rls, role, uprawnienia, grant, security definer, audyt, pgtap, kolejka, skip locked, tenant, izolacja
---

# Wytyczne: PostgreSQL jako fundament

Podejście "baza jako fundament": PostgreSQL odpowiada nie tylko za integralność danych,
ale też za granicę bezpieczeństwa (role, RLS) i te reguły domenowe, które naturalnie
należą do modelu danych. Aplikacja nie może ominąć tych mechanizmów — błąd w kodzie
backendu nie może dać szerszego dostępu do danych. Ten plik jest samodzielny — zadanie
taguj `(tech: postgres)` bez `sql` (`sql.md` opisuje klasyczny podział z logiką w aplikacji).

## Schemat i migracje

- `snake_case`, liczba mnoga dla tabel; PK `id BIGINT GENERATED ALWAYS AS IDENTITY` lub UUID
- `created_at`, `updated_at` jako `timestamptz DEFAULT now()`
- Constraints w bazie: `NOT NULL`, `UNIQUE`, `CHECK`, FK — to one są źródłem integralności, walidacja aplikacyjna to tylko UX
- Każda zmiana schematu = nowa migracja (nigdy nie edytuj starych), wersjonowana w repo
- Migracja zgodna z bieżącą ORAZ poprzednią wersją aplikacji — inaczej powrót do wcześniejszego obrazu przestaje być możliwy
- Zmiany destruktywne (DROP, utrata danych) — osobny, późniejszy krok wdrożenia
- Migracje wykonywane jako jawny etap deployu, nie automat przy starcie aplikacji

## Role i uprawnienia

- Aplikacja NIGDY jako superuser ani właściciel schematu
- Least privilege: rola dostaje wyłącznie GRANTy potrzebne do działania (per tabela, per operacja)
- Żadna rola aplikacyjna z `BYPASSRLS`; `REVOKE ALL ... FROM PUBLIC` na schemacie i funkcjach
- Osobne role dla odrębnych kontekstów (aplikacja, migracje, odczyt analityczny)

## RLS

- RLS ogranicza, KTÓRE wiersze widzi/modyfikuje organizacja, użytkownik lub kontekst — ostatnia granica dostępu do danych
- RLS NIE implementuje szczegółowych reguł biznesowych (czy można anulować wizytę, przejścia stanów, kto może wykonać operację) — te należą do warstwy domenowej, funkcji bazodanowych lub constraints, zależnie od charakteru reguły
- `ENABLE ROW LEVEL SECURITY` + `FORCE ROW LEVEL SECURITY` — polityki obowiązują też właściciela tabeli
- Polityki per operacja (SELECT/INSERT/UPDATE/DELETE), z jawnym `USING` i `WITH CHECK`
- Tożsamość użytkownika/tenanta do bazy WYŁĄCZNIE w zakresie transakcji: `SET LOCAL app.user_id = ...` — nigdy jako stan sesji/połączenia (pula połączeń przeniesie ją między żądaniami)

## Funkcje bazodanowe

- Reguła domenowa trafia tam, gdzie jest naturalna: niezmienniki → constraints; operacje wymagające atomowości i podniesionych uprawnień → funkcja bazodanowa; orkiestracja → aplikacja
- `SECURITY DEFINER` tylko gdy konieczny; wtedy obowiązkowo: `SET search_path` w definicji, w pełni kwalifikowane odwołania do obiektów, `REVOKE ALL FROM PUBLIC` + jawny `GRANT EXECUTE`
- Funkcja waliduje wejście i stan — nie zakłada, że aplikacja to zrobiła

## Audyt

- Zdarzenia audytowe → dedykowany dziennik append-only, logicznie oddzielony od logów aplikacji
- Minimum: użytkownik/sesja, rodzaj operacji, czas, obiekt operacji
- Role aplikacyjne: tylko `INSERT` (bez UPDATE/DELETE) na tabelach audytu
- Audytowany odczyt danych wrażliwych → przez kontrolowaną ścieżkę (funkcja/endpoint), która go rejestruje — triggery nie łapią SELECT

## Kolejka w PostgreSQL

- Proste zadania async: tabela zadań + `SELECT ... FOR UPDATE SKIP LOCKED LIMIT n`
- Status zadania jako kolumna (pending/processing/done/failed) + licznik prób + czas przetworzenia
- Osobny broker wiadomości dopiero, gdy rozwiązuje konkretny zmierzony problem

## Dostęp z aplikacji

- Bez ciężkiego ORM — parametryzowane zapytania przez driver (nigdy interpolacja stringów)
- Jawna lista kolumn (nie `SELECT *`); transakcje krótkie
- Liczba zapytań na operację ma znaczenie (eksperyment porównuje warianty) — nie generuj N+1

## Testy

- pgTAP dla warstwy bazy: constraints, funkcje, uprawnienia, polityki RLS
- CI stawia świeżą bazę z migracji od zera — to też jest test schematu
- OBOWIĄZKOWE testy negatywne: użytkownik/tenant NIE może odczytać ani zmodyfikować cudzych danych; test przechodzi, gdy operacja jest odrzucona lub zwraca 0 wierszy
- Testy izolacji wykonywane na roli aplikacyjnej, nie na właścicielu bazy

## Czego unikać

- Autoryzacji tylko w backendzie "bo szybciej" — granicą dostępu jest baza
- Sesyjnego `SET` do przekazywania tożsamości — tylko `SET LOCAL` w transakcji
- Roli aplikacyjnej jako właściciela tabel bez `FORCE ROW LEVEL SECURITY`
- Logiki biznesowej upchniętej w politykach RLS
- Triggerów jako mechanizmu audytu odczytów (nie działają na SELECT)
- `SELECT *`, interpolacji stringów w SQL, długich transakcji trzymających locki
