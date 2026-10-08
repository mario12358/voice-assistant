---
tag: detox
keywords: detox, e2e, mobile, react-native, ios, android, simulator, emulator, testID, gray box
---

# Wytyczne: Detox (e2e mobile React Native)

## Kiedy używać

- **Zawsze jako ostatnie zadanie fazy z UI mobile** (`tech: react-native` w tej samej fazie)
- Scenariusze od-początku-do-końca: login → nawigacja → akcja → wylogowanie
- **Nie** do testowania pojedynczych komponentów — od tego jest `@testing-library/react-native` (`tech: react-native`)
- **Nie** do testowania warstwy API bez UI — od tego są testy backendu
- Alternatywa: **Maestro** (prostszy YAML, mniej setupu, słabsza kontrola asercji) — pytaj w fazie doprecyzowania jeśli nie wiadomo

## Struktura projektu

- `e2e/` — wszystkie testy Detox (osobno od testów jednostkowych w `__tests__/`)
- `e2e/jest.config.js` — osobna konfiguracja Jest dla e2e (inny testMatch, dłuższy timeout)
- `e2e/helpers/` — fabryki danych, helpery logowania, resetowania stanu
- `e2e/<feature>.e2e.ts` — specyfikacje per feature (np. `auth.e2e.ts`)
- `.detoxrc.js` w root projektu — konfiguracja device/app per platforma

## Konfiguracja per platforma

- **iOS**: `ios.sim.debug` (Debug build na simulatorze iPhone), `ios.sim.release` dla CI
- **Android**: `android.emu.debug` (Debug APK na emulatorze), `android.emu.release` dla CI
- Build APK/IPA przed pierwszym runem: `detox build --configuration ios.sim.debug`
- **Oba** configi muszą być utrzymane — testuj na obu platformach przed ✅

## Selektory — tylko testID

React Native nie ma semantyki HTML/ARIA, więc selektor jest jeden:

```tsx
<Pressable testID="login-submit" onPress={handleLogin}>
  <Text>Zaloguj</Text>
</Pressable>
```

```typescript
await element(by.id('login-submit')).tap();
```

- **Nigdy** `by.text('Zaloguj')` jako jedyna strategia — pęka przy i18n/lokalizacji
- `by.id()` = `testID` — stabilny, wymuszony konwencją kodu
- Tekst akceptowalny tylko jako dodatkowa asercja, nie selekcja
- `testID` nadawaj hierarchicznie: `auth.login.submit`, `auth.login.email-input`

## Akcje

- `.tap()`, `.longPress()`, `.swipe('up' | 'down' | 'left' | 'right')`
- `.typeText('text')` — symuluje klawiaturę, wolniejsze ale realistyczne
- `.replaceText('text')` — szybkie ale pomija eventy `onKeyPress` (nie testuj walidacji per-znak tym)
- `.scrollTo('bottom')` / `.scroll(distance, direction)` — dla list
- `waitFor(element).toBeVisible().withTimeout(5000)` — zamiast arbitralnego sleep

## Asercje

- `await expect(element(by.id('x'))).toBeVisible()`
- `await expect(element(by.id('x'))).toHaveText('...')`
- `await expect(element(by.id('x'))).toExist()` — w DOM, niekoniecznie widoczny
- `.not.toBeVisible()` dla weryfikacji że coś zniknęło

## Czekanie

- **Nigdy** `new Promise(r => setTimeout(r, 1000))` — flaky
- `waitFor(element(by.id('x'))).toBeVisible().withTimeout(ms)` — dla asynchronicznych pojawień
- Detox synchronizuje się z RN bridge automatycznie (czeka na zakończenie animacji, requestów, timerów)
- Wyjątek: `device.disableSynchronization()` przy długich animacjach/WebView — pamiętaj włączyć z powrotem

## Reset stanu między testami

- `device.launchApp({ newInstance: true, delete: true })` w `beforeEach` dla pełnej izolacji (wolne, ale pewne)
- Albo `device.reloadReactNative()` — szybsze, resetuje tylko JS bundle
- Wyczyść AsyncStorage / SecureStore przez deep link / testowy endpoint
- Baza testowa backendu: resetowana przez helper wywołujący testowy endpoint reset (nigdy prod DB!)

## Helpery logowania

Nie loguj się UI-em w każdym teście — zbyt wolne. Zrób helper który programowo ustawia token:

```typescript
// e2e/helpers/auth.ts
export async function loginAs(user: TestUser) {
  await device.launchApp({
    newInstance: true,
    url: `myapp://test-login?token=${user.token}`, // deep link wyłącznie w __DEV__
  });
}
```

Logowanie UI-em zostawiamy dla **jednego** testu który je waliduje (`auth.e2e.ts` → login flow). Reszta scenariuszy używa helpera.

## Permissions

- `device.launchApp({ permissions: { camera: 'YES', notifications: 'YES', location: 'inuse' }})`
- Testuj też odmowę: `permissions: { camera: 'NO' }` → aplikacja musi działać w ograniczonym trybie
- iOS i Android mają różne nazwy — sprawdzaj dokumentację Detox

## Uruchamianie

- Lokalnie iOS: `detox test --configuration ios.sim.debug`
- Lokalnie Android: `detox test --configuration android.emu.debug`
- CI: `--configuration *.release`, z `--record-videos failing --record-logs failing --take-screenshots failing`
- **Przed ✅**: zielone runy na iOS sim + Android emu, nie tylko na jednej platformie

## Recording (artefakty wizualne — sekcja 0.5 instrukcji Ralph)

Gdy `ralph/config.md` ma `screeny: tak` lub `nagrania: tak`, Detox generuje artefakty per zadanie z tagiem `(screen: ...)` / `(flow: ...)`.

### Screenshoty per ekran

Zadanie z tagiem `(screen: LoginScreen)` → dodaj `device.takeScreenshot()` na końcu testu który pokazuje ten ekran:

```typescript
import { promises as fs } from 'fs';
import path from 'path';

it('Login screen — renderuje się dla niezalogowanego usera', async () => {
  await device.launchApp({ newInstance: true });
  await expect(element(by.id('login.title'))).toBeVisible();

  const tempPath = await device.takeScreenshot('LoginScreen');
  await fs.mkdir('artifacts/screens', { recursive: true });
  await fs.copyFile(tempPath, path.join('artifacts/screens', 'LoginScreen.png'));
});
```

`device.takeScreenshot(name)` zwraca ścieżkę do tymczasowego pliku w Detox artifacts; kopiuj/przenoś do `artifacts/screens/` z deterministyczną nazwą.

### Wideo per Krok

Zadanie e2e z tagiem `(flow: spec/APP_FLOW.md#krok-1)` → włącz `--record-videos all` per uruchomienie + zorganizuj output:

```javascript
// .detoxrc.js — sekcja artifacts dla flow tests
module.exports = {
  // ...
  configurations: {
    'ios.sim.debug': {
      // ...
      artifacts: {
        rootDir: 'artifacts/flows-raw',
        plugins: {
          video: { enabled: true, keepOnlyFailedTestsArtifacts: false },
          screenshot: 'manual',
        },
      },
    },
  },
};
```

Uruchom z `--record-videos all` (override config) gdy generujesz nagrania:

```bash
detox test --configuration ios.sim.debug \
  --record-videos all \
  --artifacts-location artifacts/flows-raw \
  e2e/flows/krok-1-rejestracja.e2e.ts
```

Po runie Detox produkuje wideo w `artifacts/flows-raw/<run-id>/<test-name>.mp4`. Zmień nazwę i przenieś do `artifacts/flows/`:

```typescript
// e2e/flows/krok-1-rejestracja.e2e.ts
import { promises as fs } from 'fs';
import path from 'path';
import { glob } from 'glob';

afterEach(async () => {
  const videos = await glob('artifacts/flows-raw/**/*.mp4');
  for (const video of videos) {
    await fs.mkdir('artifacts/flows', { recursive: true });
    const platform = device.getPlatform();  // 'ios' or 'android'
    await fs.rename(video, path.join('artifacts/flows', `krok-1-rejestracja-${platform}.mp4`));
  }
});

describe('krok-1: rejestracja i pierwsze logowanie', () => {
  it('happy path', async () => {
    // ...mirror prozy z spec/APP_FLOW.md#krok-1
  });
});
```

### Naming convention

- Screeny: `artifacts/screens/<id>.png` (jeden plik per id; jeśli testujesz iOS + Android, dodaj suffix `-ios` / `-android`)
- Flow wideo: `artifacts/flows/krok-<N>-<slug>-<platform>.mp4` (`<platform>` = `ios` lub `android`)
- Detox produkuje `.mp4` (nie `.webm` jak Playwright) — to OK, Ralph rozpoznaje obu

### Deterministyczność nagrań

- **Jedna platforma na nagranie** — nie miksuj iOS + Android w jednym wideo. Nagrywaj osobno, REPORT.md trzyma 2 wpisy per Krok (`-ios` i `-android`).
- **Seed bazy zawsze ten sam** dla flow tests
- **Reset stanu w `beforeEach`**: `await device.launchApp({ newInstance: true, delete: true })`
- **Wyłącz powiadomienia push** w configu testowym — nie pojawiają się w wideo z reklamami
- **Stała orientacja** (portrait dla aplikacji mobile-first; explicit jeśli landscape)

## Flaky — co robić

- Powtarzalność lokalnie 5x pod rząd = nie flaky
- Jeśli flaky na CI: zwiększ timeouty w `waitFor`, sprawdź sync (animacje, network)
- **Nigdy** `retries: 3` w configu jako workaround — flaky to bug w teście albo w aplikacji
- `device.disableSynchronization()` ostrożnie — czasem rozwiązuje, czasem maskuje prawdziwy bug

## Czego unikać

- Testowania szczegółów implementacji (nazwy klas, struktura drzewa)
- `by.text()` jako jedyna strategia selekcji
- `setTimeout` / `sleep` — używaj `waitFor`
- Współdzielonego stanu między testami (globalne zmienne, persystowana sesja)
- Logowania UI-em w każdym teście — helper z deep linkiem albo bezpośrednio token w storage
- Commitu bez uruchomienia e2e na **obu platformach** (iOS + Android)
- Testów dłuższych niż 60s — podziel scenariusz
- Hardkodowania tekstów lokalizowanych — używaj kluczy z i18n albo testID
