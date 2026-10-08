# ralph-guidelines/

Wytyczne technologiczne ładowane przez Ralph podczas realizacji zadań z ralph/PLAN.md.

## Jak to działa

1. `ralph-start.sh` kopiuje cały ten katalog do projektu przy pierwszym uruchomieniu (`project/ralph-guidelines/`)
2. Podczas generowania `ralph/PLAN.md` Claude dopasowuje do każdego zadania tagi `(tech: <tag>, ...)` na podstawie indeksu wytycznych i kontekstu zadania
3. Przy realizacji zadania Claude ładuje `general.md` + pliki dla każdego tagu
4. Wytyczne sterują stylem generowanego kodu (konwencje, struktura, testy, czego unikać)

## Format pliku

Każdy plik wytycznych ma frontmatter YAML:

```markdown
---
tag: react
keywords: react, hooks, jsx, tsx
---

# Wytyczne: React
...
```

- `tag` — **jednoznaczna nazwa** używana w ralph/PLAN.md: `(tech: react)`
- `keywords` — słowa kluczowe pomagające Claude dopasować zadania do tagów (opcjonalne ale zalecane)

Brak frontmattera = plik pominięty w indeksie.

## general.md

Plik `general.md` jest **ładowany zawsze** niezależnie od tagów w zadaniu. Zawiera uniwersalne reguły (clean code, testy, git, obsługa błędów) które dotyczą każdego języka.

## Dodawanie własnych wytycznych

1. Utwórz nowy plik `<tag>.md` w `ralph-guidelines/`
2. Dodaj frontmatter z `tag` i `keywords`
3. Opisz: konwencje, strukturę projektu, testy, czego unikać
4. Uruchom ponownie Ralph — nowy tag pojawi się w indeksie i Claude będzie go używać

## Modyfikacja istniejących

Edytuj pliki w `ralph-guidelines/` swobodnie — **Ralph NIE nadpisuje** ich przy kolejnych uruchomieniach. Twoje lokalne zmiany są źródłem prawdy dla projektu.

## Priorytety przy konflikcie

Bardziej specyficzna wytyczna wygrywa nad ogólną:

- `python.md` (język) > `general.md` (uniwersalne)
- Wytyczna projektu (edycje lokalne) > domyślny szablon z `templates/guidelines/`
- Gdy wytyczna koliduje ze specyfikacją → spec wygrywa (ale Claude powinien zapytać)

## Zachowanie przy brakującym tagu

Jeśli zadanie ma `(tech: xyz)` a pliku `xyz.md` nie ma w `ralph-guidelines/` → Claude zatrzyma się z `RALPH BLOCKED` i zapyta czy dodać wytyczną. Nie zgaduje konwencji dla nieznanej technologii.
