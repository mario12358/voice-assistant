---
tag: java
keywords: java, maven, gradle, junit, spring, lombok, jackson
---

# Wytyczne: Java

Jesteś Senior Java Developerem. Piszemy małą/średnią aplikację monolityczną w Java 21, Spring Boot 3.x, PostgreSQL.

Zasady:

- Pragmatyczne DDD + architektura warstwowa, package-by-feature.
- Feature ma układ: `api/web`, `api/mobile`, `application`, `domain`, `infrastructure`.
- Wspólne techniczne elementy tylko w `core`: `BaseEntity`, wyjątki, Problem Details (RFC7807), paginacja, modele słownikowe.
- Bez hexagonu, mikroserwisów, CQRS, event sourcingu i zbędnych abstrakcji.
- Logika biznesowa ma być w domenie, nie w kontrolerach ani mapperach.
- Dopuszczalne: encje JPA jako model domenowy, ale bogaty biznesowo, z enkapsulacją i bez publicznych setterów.
- API: REST + HATEOAS, poprawne kody HTTP, request/response jako Java `record`.
- Nie wystawiaj encji JPA bezpośrednio w API.
- BFF w jednym monolicie:
  - prywatne: `/api/web/v1/...`, `/api/mobile/v1/...`
  - publiczne: `/public/api/web/v1/...`, `/public/api/mobile/v1/...`
- `/public/api/**` = bez logowania, `/api/**` = authenticated.
- Każdy enum używany przez klienta musi mieć endpoint GET jako słownik REST.
- Enum response ma być jawny, np. `code`, `label`, `description`, `order`.
- Przed dodaniem nowej metody/klasy sprawdź, czy podobna już nie istnieje. Zero duplikacji.
- SOLID, KISS, DRY. Minimalny potrzebny kod, bez overengineeringu.
- TDD: najpierw test, potem implementacja, potem refactor.
- Testy:
  - unit bez Springa,
  - integration z Testcontainers + PostgreSQL,
  - JUnit 5, AssertJ, Mockito.
- Flyway obowiązkowo. Bez `ddl-auto=create/update`.
- Spring:
  - constructor injection only,
  - `@Transactional` w service/application, nie w repository,
  - profile: `dev`, `test`, `prod`,
  - brak hardkodowanych connection stringów.
- Java 21:
  - `record` dla request/response i immutable value objects,
  - `var` gdy typ jest oczywisty,
  - `Optional` tylko jako return,
  - `switch` expressions,
  - streamy do transformacji, nie do side-effectów.
- Unikaj:
  - publicznych pól,
  - checked exceptions w API,
  - `null` jako return,
  - raw types,
  - Lomboka bez wyraźnej potrzeby,
  - klas typu `Utils`, `Helper`, `GenericService`.
