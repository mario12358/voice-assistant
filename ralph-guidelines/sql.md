---
tag: sql
keywords: sql, database, db, mysql, sqlite, migration, schema, query, index
---

# Wytyczne: SQL / Baza danych

## Schemat

- `snake_case` dla tabel i kolumn
- Liczba mnoga dla tabel (`users`, nie `user`)
- `id` jako PRIMARY KEY (BIGSERIAL dla auto-increment lub UUID)
- `created_at`, `updated_at` jako `TIMESTAMP WITH TIME ZONE DEFAULT NOW()`
- Foreign key: `<tabela>_id` (np. `user_id`) + `FOREIGN KEY (...) REFERENCES ...`

## Migracje

- Każda zmiana schematu = nowa migracja, nigdy nie edytuj starych
- Migracja idempotentna lub bezpiecznie re-runowalna
- Nazwa: `<timestamp>_<opis>.sql` (np. `20250417120000_add_users_table.sql`)
- ZAWSZE dostarczaj rollback/down
- Migracje destruktywne (DROP, ALTER z utratą danych) — osobno, po review

## Zapytania

- Explicit column list (NIE `SELECT *` w kodzie produkcyjnym)
- `JOIN ... ON`, nie `WHERE` (stary styl)
- **Parametryzowane zapytania** zawsze — nigdy string interpolation (SQL injection)
- Indeksy dla kolumn w `WHERE` / `JOIN` / `ORDER BY` — szczególnie FK
- `EXPLAIN ANALYZE` dla wolnych zapytań

## Transakcje

- Transakcja gdy wiele zmian musi się powieść razem
- Krótkie transakcje — unikaj długich locków
- Izolacja: domyślnie READ COMMITTED, SERIALIZABLE tylko gdy potrzebne

## Integralność

- Constraints w bazie (`NOT NULL`, `UNIQUE`, `CHECK`, `FK`) — nie licz tylko na walidację aplikacyjną
- Enum → osobna tabela słownikowa lub `CHECK (status IN (...))` / typ enum (Postgres)

## Czego unikać

- `NULL` jako znaczący stan biznesowy (użyj enum / status column)
- Denormalizacja bez powodu wydajnościowego
- Trigger do logiki biznesowej (logika → aplikacja, trigger do audytu/constraintów)
- Hard delete dla encji z historią — soft delete (`deleted_at TIMESTAMP`)
- Zapisywanie haseł plain text — zawsze hash (bcrypt/argon2)
