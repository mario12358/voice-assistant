# BACKLOG — tematy na później

Lista pomysłów, drobnych ulepszeń i debt'u technicznego wyciąganych podczas
sesji interaktywnych. **Read-write** — Claude może odznaczać i archiwizować
(z potwierdzeniem usera), user edytuje swobodnie (priorytety, sortowanie,
dopisywanie ręcznie).

## Czym różni się od innych kanałów

- `changes/<plik>.md` = konkretna zmiana spec/funkcjonalności → Ralph
  aplikuje sekwencyjnie do `PLAN.md` przy następnej autonomicznej pętli
  (sekcja 0.3 instrukcji).
- `DECISIONS.md` = changelog decyzji projektowych (write-only, sekcja 10).
- `BUGFIXES.md` = changelog napraw bugów (write-only, sekcja 11).
- `REVIEWS.md` = changelog code reviewów (write-only, sekcja 12).
- **BACKLOG.md** (ten plik) = otwarte tematy, pomysły, "nice to have",
  obserwacje z dyskusji — bez gwarancji że Ralph je weźmie. User decyduje
  kiedy promować pozycję z backlogu do `changes/` albo wykonać ad-hoc.

## Format wpisu

```markdown
- [ ] **Tytuł** — `priorytet` `obszar`
  - **Kontekst**: skąd się wziął (data, wątek, obserwacja)
  - **Sugerowane rozwiązanie**: jedno-dwa zdania
  - **Zależności**: jakie inne pozycje blokują/zależą (opcjonalnie)
```

## Priorytety

- `P0` — krytyczne (do zrobienia w najbliższej sesji)
- `P1` — ważne (do zrobienia w bieżącej fazie)
- `P2` — nice to have (kiedy będzie luz)
- `P3` — pomysł na przyszłość (może nie zrobimy nigdy)

## Workflow

- **Dopisywanie** (proaktywne + potwierdzenie): Claude wyłapuje sygnały w rozmowie
  ("to na później", out-of-scope obserwacja) → proponuje wpis z priorytetem i
  obszarem → user OK / zmień priorytet / skip. Bez OK nie dopisuje.
- **Promocja BACKLOG → changes/**: gdy pozycja staje się formalną zmianą,
  user mówi "promuj X" → Claude tworzy `changes/<slug>.md` z frontmatterem
  + oznacza pozycję `[→ changes/<slug>.md]`. Następny start trybu 1 wciąga
  zmianę przez sekcję 0.3.
- **Wykonanie ad-hoc**: user mówi "zrób X" → Claude implementuje, testuje,
  commituje, oznacza `[x] (abc1234)`.
- **Archiwizacja**: po >30 dniach ukończone pozycje przenoszone do
  `BACKLOG_DONE.md` (lazy, przy starcie Ralpha gdy >10 pozycji).

Pełna procedura: sekcja 13 w `templates/instructions/INTERACTIVE.md`.

---

<!-- Dodawaj pozycje poniżej, pogrupowane w sekcje ## per obszar gdy >10 wpisów -->

## Niezakwalifikowane (do triażu)
