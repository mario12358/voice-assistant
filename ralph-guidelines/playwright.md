---
tag: playwright
keywords: playwright, e2e, end-to-end, browser, chromium, firefox, webkit, page object, fixture, trace
---

# Wytyczne: Playwright (e2e web)

## Kiedy używać

- **Zawsze jako ostatnie zadanie fazy z UI web** (`tech: react`, `vue`, `angular`, `svelte` w tej samej fazie)
- Scenariusze od-początku-do-końca z perspektywy użytkownika (login → akcja → wylogowanie)
- **Nie** do testowania pojedynczych komponentów — od tego jest RTL/Vitest (`tech: react`)
- **Nie** do testowania API bez UI — od tego jest supertest/pytest na backendzie

## Struktura projektu

- `e2e/` lub `tests/e2e/` — wszystkie testy Playwright (osobno od testów jednostkowych)
- `e2e/fixtures/` — custom fixtures (auth, seed bazy, mock API)
- `e2e/pages/` — Page Object Models (jeden plik per istotny widok)
- `e2e/specs/<feature>/` — specyfikacje per feature (np. `auth/login.spec.ts`, `auth/register.spec.ts`)
- `playwright.config.ts` w root projektu

## Selektory — kolejność preferencji

1. **Role-based**: `page.getByRole('button', { name: 'Zaloguj' })` — najbardziej odporny
2. **Label/Placeholder**: `page.getByLabel('Email')`, `page.getByPlaceholder('jan@kowalski.pl')`
3. **Text**: `page.getByText('Zapomniałeś hasła?')`
4. **Test ID**: `page.getByTestId('login-submit')` — **tylko** gdy powyższe nie wystarczą; wymagają atrybutu `data-testid` w kodzie UI
5. **CSS/XPath** — ostateczność, fragile wobec zmian stylu

## Page Object Pattern

Każdy istotny widok dostaje klasę enkapsulującą selektory i akcje:

```typescript
// e2e/pages/LoginPage.ts
export class LoginPage {
  constructor(private page: Page) {}

  readonly emailInput = this.page.getByLabel('Email');
  readonly passwordInput = this.page.getByLabel('Hasło');
  readonly submitButton = this.page.getByRole('button', { name: 'Zaloguj' });

  async goto() { await this.page.goto('/login'); }
  async login(email: string, password: string) {
    await this.emailInput.fill(email);
    await this.passwordInput.fill(password);
    await this.submitButton.click();
  }
}
```

Test używa tylko metod, nigdy nie sięga do DOM bezpośrednio.

## Fixtures — izolacja i dane testowe

- **Własny `test` z fixtures**: `import { test as base } from '@playwright/test'`
- Fixture dla zalogowanego użytkownika (unika logowania w każdym teście)
- Fixture dla seedu bazy: przed testem utwórz dane, po teście wyczyść (lub transakcja z rollbackiem)
- **Nigdy** nie dziel stanu między testami — każdy musi działać samodzielnie
- `test.beforeEach` dla setup per-test, nie `beforeAll` (współdzielony stan łamie izolację)

## Asercje

- `expect(locator).toBeVisible()` / `toHaveText()` / `toHaveURL()` — **auto-retry** (czeka do timeout)
- **Nigdy** `expect(await locator.textContent()).toBe(...)` — zamienia auto-retry w snapshot, flaky
- Custom matchers w osobnym pliku jeśli potrzebujesz domenowych asercji
- `await expect.soft(...)` dla wielu niezależnych asercji w jednym teście (nie przerywa przy pierwszym failu)

## Czekanie

- **Nigdy** `page.waitForTimeout(ms)` — arbitralny sleep, flaky
- `await page.waitForLoadState('networkidle')` — gdy czekasz na załadowanie
- `await expect(locator).toBeVisible()` — czeka aż element pojawi się w DOM
- `page.waitForResponse(url)` / `page.waitForRequest(url)` — gdy czekasz na konkretne wywołanie API

## Konfiguracja

- `playwright.config.ts`: projekty per przeglądarka (Chromium, Firefox, WebKit)
- `use.baseURL` — centralnie zdefiniowany URL aplikacji (dev server / staging)
- `use.trace: 'on-first-retry'` — trace zapisywany przy retry, pomocny do debugowania
- `use.screenshot: 'only-on-failure'`, `use.video: 'retain-on-failure'` (default; dla nagrań artefaktów → patrz sekcja Recording)
- `workers` — równoległość; ustaw na `1` dla testów piszących do tej samej bazy, inaczej schedulowany chaos
- `retries: 2` w CI, `0` lokalnie — pozwala wykryć flaky w CI bez spowalniania dev loop

## Recording (artefakty wizualne — sekcja 0.5 instrukcji Ralph)

Gdy `ralph/config.md` ma `screeny: tak` lub `nagrania: tak`, Playwright generuje artefakty per zadanie z tagiem `(screen: ...)` / `(flow: ...)`.

### Screenshoty per ekran

Zadanie z tagiem `(screen: B1_ProjectOverview)` → dodaj screenshot na końcu istniejącego testu e2e który renderuje ten ekran:

```typescript
test('Project Overview — renderuje się z fixture', async ({ page }) => {
  await page.goto('/projects/123');
  await expect(page.getByRole('heading', { name: 'Mój projekt' })).toBeVisible();
  await page.screenshot({
    path: 'artifacts/screens/B1_ProjectOverview.png',
    fullPage: true,
  });
});
```

Jeśli zadanie nie ma własnego e2e (tylko unit/integration RTL) → utwórz dedykowany krótki spec w `tests/screenshots/<id>.spec.ts` który mountuje ekran z fixture i robi screenshot. Nie używaj `@playwright/experimental-ct-react` jeśli nie jest jeszcze w projekcie — wystarczy zwykły `page.goto()` na route z fixture.

### Wideo per Krok

Zadanie e2e z tagiem `(flow: spec/APP_FLOW.md#krok-1)` → włącz nagrywanie w configu i przenieś plik do `artifacts/flows/`:

```typescript
// playwright.config.ts — config dedykowany dla flow tests
import { defineConfig } from '@playwright/test';

export default defineConfig({
  projects: [
    {
      name: 'flows',
      testMatch: /e2e\/specs\/flows\/.*\.spec\.ts/,
      use: {
        video: 'on',                     // ZAWSZE nagrywaj
        viewport: { width: 1280, height: 720 },  // deterministyczny rozmiar
        screenshot: 'off',               // nie potrzeba — wideo wystarczy
        trace: 'off',                    // czystszy artefakt
      },
      outputDir: 'artifacts/flows-raw',  // Playwright domyślnie wrzuca do test-results/
    },
    // inne projekty (chromium, firefox, ...) zostają z defaultami
  ],
});
```

Po runie Playwright zostawia wideo w `outputDir/<test-name>/video.webm`. **Zmień nazwę i przenieś do `artifacts/flows/`** (Bash w teardown lub osobnym kroku post-test):

```typescript
// e2e/specs/flows/krok-1-rejestracja.spec.ts
import { test, expect } from '@playwright/test';
import { rename, mkdir } from 'node:fs/promises';
import path from 'node:path';

test.afterEach(async ({ }, testInfo) => {
  const video = testInfo.attachments.find(a => a.contentType === 'video/webm');
  if (video?.path) {
    await mkdir('artifacts/flows', { recursive: true });
    const slug = testInfo.title.toLowerCase().replace(/[^a-z0-9]+/g, '-').slice(0, 60);
    await rename(video.path, path.join('artifacts/flows', `krok-1-${slug}.webm`));
  }
});

test('krok-1: rejestracja i pierwsze logowanie', async ({ page }) => {
  // ...mirror prozy z spec/APP_FLOW.md#krok-1
});
```

### Naming convention

- Screeny: `artifacts/screens/<id>.png` (id = jak w tagu `(screen: <id>)`)
- Flow wideo: `artifacts/flows/krok-<N>-<slug>.webm` (slug = title kebab-case, max 60 znaków)
- Nie commituj `artifacts/` jeśli `commit artefakty: nie` w configu — sprawdź `.gitignore`

### Deterministyczność nagrań

- **Viewport fixed** (`1280x720` lub `1920x1080`) — bez tego ten sam test produkuje różne wideo per maszyna
- **Wyłącz animacje** dla flow tests jeśli rozpraszają wizualnie: `await page.emulateMedia({ reducedMotion: 'reduce' })`
- **Mockuj timestampy** jeśli pojawiają się w UI: `await page.addInitScript(() => Date.now = () => 1700000000000)`
- **Seed bazy zawsze ten sam** dla flow tests — wideo ma demonstrować zachowanie, nie różne dane

## Uruchamianie

- Lokalnie: `npx playwright test` (headless) lub `--headed` do debugu
- Debug: `npx playwright test --debug` — Inspector z step-through
- UI mode: `npx playwright test --ui` — interaktywny podgląd
- CI: headless, z trace/video dla failów
- **Przed oznaczeniem zadania e2e jako ✅**: uruchom `npx playwright test` i zobacz green na wszystkich przeglądarkach z configu

## Seed bazy / stan startowy

- Dedykowana baza testowa (nie dev, nie prod) — izolacja
- Globalny setup (`globalSetup` w config) migruje schemat + seed bazowych danych (role, słowniki)
- Fixture per-test dodaje dane specyficzne dla scenariusza
- Po całym runie: drop bazy testowej albo zostaw w ostatnim dobrym stanie

## Mockowanie / stubbing

- Zewnętrzne API (Stripe, SendGrid, SMS): `page.route('**/api.stripe.com/**', route => route.fulfill({...}))`
- **Nie mockuj** własnego backendu — e2e ma sprawdzić integrację front-back
- Wyjątek: eventy asynchroniczne (webhooks, queues) — mockuj jeśli nieprzewidywalne w testach

## Czego unikać

- Testowania szczegółów implementacji (klasy CSS, struktura DOM)
- `page.locator('.css-abc123')` — wygenerowane klasy, fragile
- Długich testów testujących wszystko naraz (1 scenariusz = 1 test, max 30s)
- Współdzielonego stanu między testami (żadne globalne zmienne)
- `console.log` w testach — użyj `test.step('nazwa', async () => { ... })` dla struktury raportu
- Pomijania testów przez `test.skip` bez komentarza z powodem + datą review
- Commitu testów e2e bez uruchomienia ich lokalnie na GREEN
