---
tag: react
keywords: react, jsx, tsx, hooks, component, state, props, vite
---

# Wytyczne: React

## Komponenty

- Funkcyjne z hookami — NIE class components
- Nazwa pliku = nazwa komponentu, PascalCase (`UserList.tsx`)
- Jeden komponent na plik (plus tightly coupled sub-components)
- Props typowane interfejsem: `interface Props { ... }` (TypeScript)

## Struktura

- `src/components/` — współdzielone, prezentacyjne komponenty
- `src/features/<nazwa>/` — feature-bazowa organizacja: komponent + hook + test + style
- `src/hooks/` — custom hooki współdzielone między featurami
- `src/lib/` — utility/helpers bez zależności od React

## Hooki

- `useEffect` ZAWSZE z listą zależności (nigdy bez — powoduje pętle)
- Custom hook dla logiki reużywalnej (prefix `use`)
- Nie wywołuj hooków warunkowo / w pętlach (rules of hooks)
- `useMemo` / `useCallback` tylko gdy rzeczywisty perf problem, nie prewencyjnie

## Stan

- `useState` dla lokalnego, Context dla cross-component, store (Redux/Zustand) dla globalnego
- Prop drilling > 2 poziomy → context lub store
- Stan unoszony do najniższego wspólnego przodka
- Form state: `react-hook-form` > ręczny `useState` per pole

## Testy

- React Testing Library + Vitest (preferowane) lub Jest
- Testuj zachowanie (co user widzi/klika), nie implementację
- `screen.getByRole` > `getByTestId` > `querySelector`
- `userEvent` > `fireEvent` (symuluje realne interakcje)

## Wydajność

- `React.memo` tylko gdy profilowanie pokaże potrzebę
- Wirtualizacja dla list > 100 elementów (`react-window`)
- Lazy loading tras: `React.lazy` + `Suspense`

## Praca z mockupami z `spec/ux_ui/`

Jeśli zadanie ma tag `(ui-tokens: ...)`, `(ui-component: ...)` lub `(ui-screen: ...)`, pliki w `spec/ux_ui/` są **referencją designu**, nie kodem do importu. Procedura ładowania w sekcji 0.4 instrukcji.

- **Tokens (`tokens.css`)** — kopiuj 1:1 do `src/styles/tokens.css`, podłącz globalnie (`import './styles/tokens.css'` w `main.tsx` lub root komponencie). Klasy `theme-dark` / `theme-light` na `<html>` lub root `<div>`.
- **Komponenty (`*.jsx`)** — to prototypy w Babel-in-browser (globalne `window.X`, brak importów ES modules, JSX bez TypeScript). **NIE kopiuj 1:1** — przepisz każdy komponent jako osobny plik `src/components/<Nazwa>.tsx`:
  - `interface Props { ... }` zamiast nieotypowanych props
  - Importy zamiast globalnych zmiennych
  - Nazwany default export lub named export — spójnie w projekcie
  - Style przez `var(--*)` z `tokens.css` — nigdy hardcoded hex/px gdy istnieje token
- **Ekrany (`screens-*.jsx`)** — implementuj jako `src/screens/<Nazwa>.tsx` (lub `src/features/<Nazwa>/<Nazwa>Screen.tsx`). Używaj komponentów które są już w `src/components/` (z poprzednich tasków setupu) — NIE wracaj do prototypu po API komponentów, czytaj produkcyjne pliki.
- **Zachowanie wiernie z mockupu**: layout, hierarchia, stany (hover/active/disabled/loading/error), animacje, interakcje. Pixel-aware — wymiary z mockupu zachowuj.
- **Brak elementu w SPEC_INDEX** lub niedopasowane line range → RALPH BLOCKED. Nie zgaduj wyglądu.

## Czego unikać

- `any` w TypeScript
- Logika biznesowa w komponentach (wydzielaj do hooków/usług)
- Inline style gdy masz system (CSS modules / styled-components / Tailwind)
- `dangerouslySetInnerHTML` bez sanityzacji (DOMPurify)
- `index` jako `key` w listach dynamicznych
- **Kopiowanie kodu z `spec/ux_ui/*.jsx` 1:1** — zawsze przepisuj do TS + ES modules
- **Hardcoded kolory/odstępy** gdy odpowiedni token istnieje w `tokens.css`
