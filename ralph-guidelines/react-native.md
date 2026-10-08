---
tag: react-native
keywords: react-native, rn, expo, android, ios, mobile, native, flatlist, stylesheet, navigation
---

# Wytyczne: React Native

## Komponenty

- Funkcyjne z hookami — NIE class components
- Nazwa pliku = nazwa komponentu, PascalCase (`LoginScreen.tsx`)
- Jeden komponent na plik (plus tightly coupled sub-components)
- Props typowane interfejsem: `interface Props { ... }` (TypeScript)
- Używaj natywnych prymitywów (`View`, `Text`, `Pressable`) — nie HTML (`div`, `span`)
- **Nigdy** nie renderuj gołego stringa poza `<Text>` (crash na Android)

## Struktura projektu

- `src/screens/` — ekrany (pełne widoki spinane z nawigacją)
- `src/components/` — współdzielone komponenty prezentacyjne
- `src/navigation/` — konfiguracja React Navigation (stack, tabs, drawer)
- `src/features/<nazwa>/` — feature-bazowa organizacja: ekran + hook + test + style
- `src/hooks/` — custom hooki współdzielone między featurami
- `src/lib/` — utility/helpers bez zależności od React
- `src/services/` — klient API, storage, push notifications

## Nawigacja

- `@react-navigation/native` jako standard — stack / bottom-tabs / drawer per potrzeba
- Typowanie tras: `ParamList` dla każdego nawigatora + `NativeStackScreenProps<ParamList, 'Screen'>`
- Deep linking konfigurowany w `linking` na `NavigationContainer`
- Nie trzymaj stanu aplikacji w params trasy — tylko identyfikatory / filtry

## Stan

- `useState` dla lokalnego, Context dla cross-screen, store (Redux Toolkit / Zustand) dla globalnego
- Server state: `@tanstack/react-query` — nie rób ręcznie cache w `useState`
- Persistencja: `@react-native-async-storage/async-storage` (nigdy nie zapisuj tam tokenów bez szyfrowania)
- Secrets / tokeny: `expo-secure-store` (iOS Keychain, Android Keystore)

## Stylowanie

- `StyleSheet.create({ ... })` zamiast obiektów inline (walidacja, minimalny perf boost)
- Alternatywnie: `nativewind` (Tailwind) lub `tamagui` — **jedna konwencja per projekt**
- Jednostki: `dp` domyślnie (liczby bez `px`); procenty dla elastycznych layoutów
- Safe area: `react-native-safe-area-context` + `SafeAreaView` lub `useSafeAreaInsets`
- Unikaj fixed heights — używaj `flex`, `flexDirection`, `gap`

## Listy i wydajność

- `FlatList` / `SectionList` dla dynamicznych list — **nigdy** `ScrollView` + `map` dla >20 elementów
- `keyExtractor` zawsze ustawiony (stabilne ID, nie `index`)
- `getItemLayout` jeśli wysokość elementów jest stała (duży boost)
- `React.memo` na item-component + stabilne callbacki (`useCallback`)
- Obrazy: `expo-image` lub `react-native-fast-image` (cache, progressive loading)

## Platform-specific

- `Platform.OS === 'ios'` / `'android'` dla rozgałęzień
- Pliki `*.ios.tsx` / `*.android.tsx` dla dużych różnic — bundler sam wybierze wariant
- Sprawdzaj na **obu platformach** przed oznaczeniem zadania ✅ — RN nie gwarantuje pixel-parity
- iOS: testuj na symulatorze + realnym urządzeniu (permissions, haptics)
- Android: testuj z `enableHermes: true` (domyślnie) + uwzględnij back button (`BackHandler`)

## Formularze

- `react-hook-form` + `@hookform/resolvers/zod` — NIE ręczny `useState` per pole
- Walidacja: Zod schema współdzielona z backendem jeśli możliwe
- Klawiatura: `KeyboardAvoidingView` (iOS: `padding`, Android: `height`) + `react-native-keyboard-controller` dla złożonych przypadków
- `keyboardType`, `autoCapitalize`, `autoCorrect`, `returnKeyType` — zawsze świadomie ustawione

## API i sieć

- `fetch` lub `axios` — jeden wybór per projekt
- Timeout zawsze ustawiony (domyślny fetch nie ma timeoutu — użyj `AbortController`)
- Retry z backoffem dla błędów 5xx / sieciowych (react-query to robi)
- Offline: `@react-native-community/netinfo` do wykrywania; queue mutacji dla krytycznych akcji

## Testy

- Jest + `@testing-library/react-native` — testuj zachowanie, nie implementację
- `screen.getByRole` / `getByText` > `getByTestId`
- `userEvent` z `@testing-library/user-event` > `fireEvent`
- E2E: **Detox** (preferowane) lub **Maestro** — testuj krytyczne flow (login, checkout)
- Mockuj natywne moduły przez `jest.mock` w `jest-setup.ts`

## Permissions i uprawnienia

- Proś o uprawnienie **w momencie potrzeby** (nie wszystkie na starcie)
- Wyjaśnij cel w `Info.plist` (iOS) i `AndroidManifest.xml` (Android) — wymagane przy store review
- Obsłuż odmowę — aplikacja musi działać w ograniczonym trybie, nie crashować

## Praca z mockupami z `spec/ux_ui/`

Jeśli zadanie ma tag `(ui-tokens: ...)`, `(ui-component: ...)` lub `(ui-screen: ...)`, pliki w `spec/ux_ui/` są **referencją designu**, nie kodem do importu. Procedura ładowania w sekcji 0.4 instrukcji.

**Specyfika React Native** (różni się od web-Reactowych mockupów):

- Mockupy są zwykle webowe (HTML/CSS/JSX z `<div>`, `<svg>`, `style={{...}}`) — w RN **musisz przepisać** na natywne prymitywy:
  - `<div>` → `<View>`, `<span>` / tekst → `<Text>` (nigdy goły string poza `<Text>` — crash na Android)
  - `<button>` / `onClick` → `<Pressable>` / `onPress`
  - `<svg>` z mockupu → `react-native-svg` (`<Svg>`, `<Path>`)
- **Tokens (`tokens.css`)** — CSS Variables NIE działają w RN. Przepisz `tokens.css` na `src/styles/tokens.ts`:
  ```ts
  export const tokens = {
    colors: { accent: '#6B6BF2', /* ... */ },
    fonts: { sans: 'IBMPlexSans', mono: 'JetBrainsMono' },
    /* ... */
  };
  export const theme = {
    dark: { bgSurface: '#0E0F13', textPrimary: '#ECEDF1', /* ... */ },
    light: { bgSurface: '#FAFAF7', textPrimary: '#18181B', /* ... */ },
  };
  ```
  Theme przez Context (`ThemeProvider`) + `useTheme()` hook. Lub `nativewind` jeśli stack używa Tailwinda — wtedy generuj `tailwind.config.js` z mappingu tokenów.
- **Komponenty (`*.jsx`)** — przepisz do `src/components/<Nazwa>.tsx` używając `View`/`Text`/`Pressable`. Style przez `StyleSheet.create({...})` z odwołaniem do `theme` (lub `nativewind` classes).
- **Ekrany (`screens-*.jsx`)** — implementuj jako `src/screens/<Nazwa>.tsx`, podłącz do nawigatora. Używaj komponentów już istniejących w `src/components/`.
- **Pixel mismatches**: wymiary z mockupu webowego (np. 44px topbar) traktuj jako `dp` w RN. Sprawdzaj na obu platformach (iOS + Android) — RN nie gwarantuje pixel-parity z webem.
- **Fonty z tokens.css** — bundluj w `assets/fonts/`, zarejestruj przez `expo-font` (Expo) lub `react-native.config.js` (bare RN). Nie zakładaj że `IBM Plex Sans` jest dostępny systemowo.
- **Brak elementu w SPEC_INDEX** lub niedopasowane line range → RALPH BLOCKED. Nie zgaduj wyglądu.

## Czego unikać

- `any` w TypeScript
- Logika biznesowa w ekranach (wydzielaj do hooków / serwisów)
- **Kopiowanie kodu z `spec/ux_ui/*.jsx` 1:1** — to webowy JSX (`<div>`, CSS variables), w RN trzeba przepisać na natywne prymitywy
- **CSS variables (`var(--*)`) w stylach RN** — nie działają, używaj theme/Context lub nativewind
- `console.log` w kodzie produkcyjnym (używaj `__DEV__` guard lub bibliotek typu Flipper / Reactotron)
- Inline styles gdy masz `StyleSheet` / nativewind
- `Dimensions.get('window')` bez listenera (nie reaguje na rotację / split-screen) — użyj `useWindowDimensions`
- Zależności natywne dodawane bez `pod install` (iOS) / rebuilda (Android) — zgłaszaj RALPH BLOCKED jeśli wymagane
- Commity bez przetestowania na **obu platformach**
