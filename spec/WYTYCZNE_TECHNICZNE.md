# Wytyczne techniczne projektu

> ## Jak korzystać
>
> **Wypełnij TYLKO pola w których odstępujesz od defaultów z `ralph-guidelines/`.**
> Puste pole = użyj defaulta. Nie wpisuj „React" tylko dlatego że i tak go używasz —
> defaulty z guidelines załatwią sprawę. Wpisz tylko gdy: klient narzuca, infrastruktura
> wymaga, regulacje wymuszają, zespół zna tylko X.
>
> **Wymagania biznesowe i funkcjonalne** (use case'y, model domeny, kryteria akceptacji,
> mockupy, diagramy) dorzuć jako osobne pliki w `spec/` — w dowolnym formacie.
> Claude przeczyta wszystkie pliki w katalogu.
>
> **Niezdecydowane**: zostaw puste albo wpisz `?`. Claude wygeneruje pytania w `ralph/QUESTIONS.md`.
>
> Po wypełnieniu usuń marker `RALPH_SPEC_TEMPLATE` z pierwszej linii pliku.

---

## Stack — odstępstwa od defaultów

<!-- Wypełnij gdy narzucone z zewnątrz. Inaczej zostaw puste. -->

- **Język / runtime**: Rust <!-- np. „Java 17 — klient ma zespół Javowy" -->
- **Framework backend**: <!-- np. „Spring Boot — firmowy szablon startowy" -->
- **Framework frontend**: <!-- np. „Angular 17 — istniejąca aplikacja" -->
- **Baza danych**: <!-- np. „Oracle 19c — istnieje, nie zmieniamy" -->
- **ORM / dostęp do DB**: <!-- np. „bez ORM — raw SQL" / „MyBatis" -->
- **Test runner**: <!-- np. „Jest — reszta firmy go używa" -->
- **Inne narzucone biblioteki**: model large-v3-turbo <!-- np. „firmowa auth-lib v3" -->

## Hosting i infrastruktura

- **Środowisko docelowe**: <!-- np. „on-prem RHEL 8 bez Dockera" / „AWS Lambda" / „Vercel" / „Kubernetes (GKE)" -->
- **Containery**: <!-- np. „bez Dockera" / „Docker + docker-compose" / „k8s manifesty" -->
- **CI/CD**: <!-- np. „GitLab CI (firmowy runner)" / „GitHub Actions" / „brak — deploy ręczny" -->
- **Sekrety / config**: <!-- np. „Vault" / „AWS Secrets Manager" / „env vars" -->
- **Monitoring / logi**: <!-- np. „Datadog" / „ELK" / „Grafana + Loki" -->

## Integracje zewnętrzne

<!-- Każdy system zewnętrzny z którym musimy rozmawiać. Pomiń sekcję jeśli brak. -->

### System: [nazwa]

- **Protokół**: <!-- REST / SOAP / GraphQL / Kafka / RabbitMQ / SFTP -->
- **Auth**: <!-- API key / OAuth2 / mTLS / Basic / brak -->
- **Endpointy / docs**: <!-- link do Swaggera / lista operacji -->
- **SLA / limity**: <!-- rate limit, godziny dostępności -->
- **Środowiska**: <!-- URL prod / staging / sandbox + dane testowe -->

## Wymagania niefunkcjonalne (gdy odbiegają od defaultów)

<!-- Wpisuj TYLKO gdy spec biznesowy lub regulator narzuca konkretną wartość. -->

- **Wydajność**: <!-- np. „p95 < 200 ms dla GET /search" / „import 1M wierszy < 5 min" -->
- **Skalowalność**: <!-- np. „100k aktywnych użytkowników, 10 GB danych dziennie" -->
- **Dostępność (SLA)**: <!-- np. „99.9% w godz. 6:00–22:00" -->
- **Compliance**: <!-- np. „RODO — pseudonimizacja PII, retencja 5 lat" / „PCI-DSS" / „HIPAA" -->
- **Bezpieczeństwo (override defaultów)**: <!-- np. „MFA wymagane dla adminów" / „TLS 1.3 only" -->
- **Lokalizacja / i18n**: <!-- np. „PL + EN, strefa Europe/Warsaw, format dat dd.MM.yyyy" -->
- **Dostępność (a11y)**: <!-- np. „WCAG 2.1 AA — wymóg ustawowy" -->
- **Browser support**: <!-- np. „mobile Safari 14+" / „Chrome ostatnie 2 wersje" -->
- **Offline**: <!-- np. „PWA z service workerem, sync gdy online" -->

## Inne ograniczenia

<!-- Cokolwiek istotnego co nie pasuje wyżej. Pomiń sekcję jeśli nic. -->

- model ma działać tylko na GPU

---

## Wymagania biznesowe i funkcjonalne

**Nie wpisuj tutaj** — dorzuć osobne pliki do `spec/` z opisem:

- celu i kontekstu projektu
- use case'ów / user stories
- modelu domenowego
- reguł biznesowych
- kryteriów akceptacji
- mockupów / diagramów (jako pliki graficzne lub w `spec/diagrams/`)
- słownika pojęć

Format dowolny (.md, .txt, .pdf, .docx — pliki binarne są auto-konwertowane gdy `pandoc` jest zainstalowany). Claude w fazie clarification przeczyta wszystkie pliki w `spec/`.
