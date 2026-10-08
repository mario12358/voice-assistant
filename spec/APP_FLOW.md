<!-- RALPH_APP_FLOW_TEMPLATE: usuń tę linię gdy zaczniesz wypełniać szablon -->

# APP_FLOW.md — Procesy aplikacji (scenariusze user → system)

> **Po co ten plik?**
> Opisuje **co user robi krok po kroku i jak system reaguje** — z perspektywy użytkownika, bez detali technicznych.
> Każdy `## Krok N — ...` to **osobny scenariusz e2e + jedno nagranie wideo** (mapowanie 1:1).
>
> **Plik jest opcjonalny.** Tworzony tylko jeśli w `ralph/config.md` ustawisz `nagrania: tak` w sekcji
> `## Artefakty wizualne`. Bez tego pliku Ralph nie nagrywa wideo (info w REPORT.md, nie blokada).
>
> Po wypełnieniu USUŃ marker `RALPH_APP_FLOW_TEMPLATE` z pierwszej linii pliku.

---

## Zasady wypełniania

- **Jeden `## Krok N — Tytuł` = jedno nagranie wideo.** Granularność precyzyjna: jeden Krok = jeden user flow od początku do końca (rejestracja, logowanie, utworzenie obiektu, itp.). Nie składaj wielu niepowiązanych akcji w jeden Krok.
- **Tytuł Kroku** krótki, opisowy (np. `Rejestracja i utworzenie projektu`, `Multi-upload specyfikacji`). Trafia jako identyfikator w tagach `(flow: APP_FLOW.md#krok-N)` w `ralph/PLAN.md`.
- **Pisz z perspektywy usera**: *„User klika X. System pokazuje Y."* — nie *„endpoint POST /api/foo zwraca JSON"*. Cele technicze opisz w innych dokumentach `spec/` (api-spec.md, db-schema.md, ...).
- **Wymień konkretne UI/interakcje**: nazwy przycisków, etykiety pól, komunikaty błędów, status badge'y. To są elementy które test e2e będzie wyszukiwał (`getByRole`, `getByText`, `testID`).
- **Podścieżki** (Ścieżka A / Ścieżka B / itp.) opisuj **w obrębie tego samego Kroku** — to wciąż jeden test e2e który może mieć wewnątrz `test.describe()` z wariantami. Nie rozdzielaj na osobne Kroki, chyba że to naprawdę różne user flows.
- **Stany błędów/edge case'y** — wymień te które chcesz pokryć testem (np. *„jeśli email zajęty → komunikat: Email jest już zarejestrowany"*). To stanie się dodatkową asercją w teście.
- **Persony** (jeśli aplikacja ma role) — wymień raz na początku pliku, potem odwołuj się przez imię/rolę.

---

## Persony (przykład — dostosuj do projektu lub usuń)

- **Admin** — pełen dostęp, tworzy projekty, zarządza userami
- **User** — standardowy użytkownik, korzysta z aplikacji
- **Guest** — niezalogowany, ograniczony dostęp

---

## Krok 1 — Rejestracja i pierwsze logowanie

**User**: wchodzi na stronę główną, klika **"Zarejestruj się"**. Wypełnia formularz (email, hasło, powtórz hasło), klika **"Załóż konto"**.

**System**:
- Waliduje pola (email format, hasło ≥ 8 znaków, hasła muszą być zgodne)
- Jeśli email zajęty → komunikat: *"Email jest już zarejestrowany"* (czerwony pod polem email)
- Jeśli OK → tworzy konto, wysyła email weryfikacyjny, pokazuje ekran *"Sprawdź skrzynkę"* z przyciskiem **"Wyślij ponownie"**

**User**: klika link z emaila → ląduje na ekranie logowania z toast'em *"Email potwierdzony"*. Wpisuje dane i klika **"Zaloguj"**.

**System**: weryfikuje credentials, ustawia sesję, redirect na **dashboard** z powitaniem.

### Edge case'y do pokrycia testem

- Email niepoprawnego formatu → komunikat *"Niepoprawny format email"*
- Hasło za krótkie → komunikat *"Hasło musi mieć min. 8 znaków"*
- Hasła nie zgadzają się → komunikat *"Hasła nie są identyczne"*
- Próba logowania bez weryfikacji emaila → komunikat *"Potwierdź email zanim się zalogujesz"*

---

## Krok 2 — Utworzenie pierwszego obiektu (projekt / dokument / cokolwiek)

**User**: na dashboardzie klika **"+ Nowy projekt"**. Modal z polami: nazwa (wymagane), opis (opcjonalne), kategoria (dropdown).

**User** wpisuje nazwę *"Mój pierwszy projekt"*, wybiera kategorię, klika **"Utwórz"**.

**System**:
- Tworzy projekt z generowanym ID
- Redirect na ekran projektu (`/projects/<id>`)
- Toast: *"Projekt utworzony"*
- Lista projektów na dashboardzie aktualizuje się przy następnym wejściu

### Ścieżka A: utworzenie pustego projektu

User pomija opcjonalne pola, klika **"Utwórz"** od razu po wpisaniu nazwy — projekt powstaje, ekran projektu pokazuje CTA *"Dodaj pierwszą zawartość"*.

### Ścieżka B: utworzenie projektu z opisem i kategorią

User wypełnia wszystkie pola — ekran projektu pokazuje opis pod nazwą + badge kategorii.

### Edge case'y

- Nazwa pusta → przycisk **"Utwórz"** disabled
- Nazwa > 100 znaków → komunikat *"Nazwa za długa (max 100)"*

---

## Krok 3 — [Tytuł kolejnego procesu]

**User**: [co user robi]

**System**: [co system robi]

[...kolejne kroki według potrzeb projektu — typowo 5-12 Kroków na całą aplikację]

---

## Notatki dla Ralpha (czytaj jako AI)

- Każdy `## Krok N — ...` to **osobny test e2e** z tagiem `(flow: APP_FLOW.md#krok-N)` w PLAN.md. Mapowanie tagu na fragment pliku odbywa się przez `ralph/SPEC_INDEX.md` sekcja `## Procesy` (line range per Krok).
- Implementując test, traktuj prozę jako scenariusz: *"User wpisuje X w pole Y"* → `await page.getByLabel('Y').fill('X')`.
- Edge case'y z sekcji `### Edge case'y do pokrycia testem` muszą trafić jako osobne `test()` lub `test.describe()` w tym samym pliku.
- Nazwa pliku nagrania: `artifacts/flows/krok-N-<slug>.<ext>` (slug = tytuł kroku w kebab-case, np. `krok-1-rejestracja-i-pierwsze-logowanie.webm`).
