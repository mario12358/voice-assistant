---
tag: typescript
keywords: typescript, types, interfaces, generics, tsconfig, strict
---

# Wytyczne: TypeScript

## Konfiguracja

- `"strict": true` w `tsconfig.json` — obowiązkowo
- `"noUncheckedIndexedAccess": true`
- `"noImplicitReturns": true`
- Target >= ES2020
- W projekcie Deno: te same opcje w `compilerOptions` w `deno.json` — NIE twórz osobnego `tsconfig.json`

## Typy

- `interface` dla obiektów / kontraktów API, `type` dla unii / intersekcji / aliasów
- Preferuj `unknown` nad `any` (wymusza narrowing)
- Narrowing przez type guards (`function isUser(x): x is User`) — nie rzutowanie `as`
- Wyczerpujący `switch` z `never` exhaustive check:
  ```ts
  const _exhaustive: never = variant
  ```

## Nazewnictwo

- PascalCase dla typów/interfejsów (bez prefiksu `I` — to C# konwencja)
- camelCase dla zmiennych/funkcji
- Sufiksy semantyczne: `Props`, `State`, `Config`, `Dto`

## Generyki

- Używaj gdy typ zależy od argumentu
- Jednoznaczne nazwy (`TItem`, `TKey`) zamiast `T`, `U` dla > 1 generyka
- Constraint (`T extends X`) gdy wymagasz cech — nie ogólne `T`

## Moduły

- `import type { ... }` dla importów czysto typowych (łatwiejszy tree-shaking)
- Bariera barrel files (`index.ts` z re-exports) rozważnie — łatwo robią cycle imports

## Czego unikać

- `any` — jeśli musisz, `unknown` + narrowing
- Non-null assertion `!` — preferuj sprawdzenie + early return
- `@ts-ignore` — użyj `@ts-expect-error` z komentarzem dlaczego
- `enum` — preferuj union types (`type Role = "admin" | "user"`) lub `as const` obiekt
- Nadmierne generyki gdy konkretny typ wystarczy
