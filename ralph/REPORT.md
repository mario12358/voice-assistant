# ralph/REPORT.md - Raport realizacji projektu

<!-- Plik trafia W CAŁOŚCI do promptu startowego każdej sesji. Dwa rodzaje sekcji:            -->
<!--   • stan bieżący (Podsumowanie) — NADPISUJ, to ma być jeden aktualny opis;             -->
<!--   • kronika (Stan testów, Historia realizacji, Historia zmian, Log problemów,           -->
<!--     Artefakty) — DOPISUJ jeden wpis; starsze rotują się do ralph/REPORT_ARCHIVE.md      -->
<!--     przy starcie (Stan testów: 3 ostatnie przebiegi).                                   -->
<!-- Nie zakładaj własnych sekcji `## ` na podsumowania sesji — KAŻDA sekcja `## ` spoza      -->
<!-- tego szablonu jest doraźna i zostają z nich tylko 3 najnowsze.                          -->
<!-- Rotacja przenosi CAŁE wpisy, także wielolinijkowe. Wpis musi mieć rozpoznawalny         -->
<!-- początek (wiersz tabeli, `### …`, `- [data] …`, `- **Nazwa …**`), a akapit doklejony     -->
<!-- pod wpisem liczy się jako jego ciąg dalszy i pojedzie do archiwum razem z nim.           -->

## Podsumowanie

- **Start**: 2026-10-08
- **Status**: W trakcie
- **Postęp**: 11/47 pozycji ukończonych
- Specky: pracuję jako mariusz.iskra (mariusz.iskra@gmail.com), organizacja mariusz.iskra's Organization. Synchronizacja 2026-10-08 (kursor 704): 0 zmian, kolejka pusta; VA-MODEL-1 czeka na decyzję. Na prośbę właściciela VA-PLAT-1 i VA-TECH-1 oznaczone `code_ready`.
- Otwarte PR: #1 (2.3)

## Stan testów

<!--
Tabela to stan bieżący — PODMIENIAJ liczby, nie dokładaj drugiej tabeli.
Nad tabelą jeden akapit per przebieg pełnego suite'u, otwarty pogrubieniem z datą:

**Regresja po Fazie 12 (2026-08-20) — ZIELONA:** backend `pytest -q` 340/0, frontend 210/0.
Względem 328/205 po Fazie 11: +12 testów backendu. Tag `ralph/faza-12`.

Data w pierwszej linii jest OBOWIĄZKOWA — po niej rotacja rozpoznaje, które przebiegi
zostawić (3 ostatnie), a które przenieść do ralph/REPORT_ARCHIVE.md. Akapit bez daty
jest traktowany jako ciąg dalszy przebiegu powyżej. Bez daty blok zostaje w pliku
na zawsze i rośnie w prompcie każdej sesji.
-->

**Regresja po Fazie 1 (2026-10-08) — ZIELONA:** `cargo test --workspace -- --include-ignored` 17/0 (0 ignored), clippy `-D warnings` czysto, `cargo fmt --check` czysto. Tag `ralph/faza-1`.

| Metryka | Wartość |
|---------|---------|
| Łącznie testów | 43 |
| Pass | 41 |
| Fail | 0 |
| Skip | 2 (ignored: mikrofon) |
| Ostatnie uruchomienie | 2026-10-08 (cargo test --workspace po 2.3) |

## Historia realizacji

| Zadanie | Status | Testy | Próby | Commit | Czas |
|---------|--------|-------|-------|--------|------|
| 1.1 Architektura i konwencje | ✅ | n/d (dokument) | 1 | dd3d2cc | 2026-10-08 |
| 1.2 Workspace Cargo + test 1.2 | ✅ | 2/0 | 1 | 70a8fe2 | 2026-10-08 |
| 1.3 Konfiguracja TOML + test 1.3 (`va-dev config`, start aplikacji) | ✅ | 14/0, mutacja numeru linii czerwona | 1 | 99d8692 | 2026-10-08 |
| 1.4 Logowanie tracing + test 1.4 | ✅ | 17/0, mutacja info→treść czerwona | 1 | 229aad4 | 2026-10-08 |
| 2.1 Lista mikrofonów, wybór z fallbackiem, `va-dev devices` + test 2.1 | ✅ | 23/0 (z ignored), mutacja fallbacku czerwona | 1 | cc5599a | 2026-10-08 |
| 2.2 Nagrywanie do bufora → 16 kHz mono (rubato Fft) + test 2.2, strażnik „bez zapisu na dysk” | ✅ | 33/0 + 2 ignored (nagranie z mikrofonu zielone lokalnie), mutacja limitu czerwona | 1 | 246b3b1 | 2026-10-08 |
| 2.3 Przycinanie ciszy (VAD RMS, ramki 20 ms) + test 2.3 na fixtures WAV | ⏳ PR #1 | 41/0 + 2 ignored, mutacja progu czerwona (3 testy) | 1 | 8fa59f4 (gałąź) | 2026-10-08 |

## Historia zmian

<!--
Wpisy dodawane przez Ralpha po przetworzeniu pliku z changes/.
Format wpisu (jeden ### na każdą zmianę):

### YYYY-MM-DD <nazwa-pliku>.md
- **Rodzaj**: błąd | korekta | rozwój
- **Źródło błędu**: [tylko przy błędzie: zadanie X.Y, które to zbudowało | nieznane]
- **Wykryte przez**: [tylko przy błędzie: test | scenariusz | review | ralph | właściciel | produkcja]
- **Priorytet**: krytyczny | normalny
- **Dotyczy**: [pole dotyczy z frontmattera lub "ogólne"]
- **Streszczenie**: [1-2 zdania co się zmienia i dlaczego]
- **Zmodyfikowane zadania pending**: [lista numerów, np. 4.2, 5.1] lub "brak"
- **Nowa faza rework**: [np. Faza 6.Z1 z 3 zadaniami] lub "brak"
- **Nowe zadania**: [lista numerów, np. 7.4, 7.5] lub "brak"

Plik źródłowy zmiany żyje w changes/processed/<nazwa-pliku>.md (audyt w gicie).
-->

## Artefakty wizualne

<!--
Sekcja uzupełniana TYLKO gdy w ralph/config.md sekcja `## Artefakty wizualne` ma
`screeny: tak` lub `nagrania: tak`. Pełna procedura: sekcja 0.5 instrukcji.

Format wiersza:
| Faza | Typ | ID | Plik artefaktu | Spec source |
|------|-----|----|--------------:|-------------|
| 1    | screen | B1_ProjectOverview | artifacts/screens/B1_ProjectOverview.png | spec/ux_ui/screens-b1-b2.jsx:52-200 |
| 1    | flow   | krok-1             | artifacts/flows/krok-1-rejestracja.webm  | spec/APP_FLOW.md:11-30 |

Info-warningi (brak APP_FLOW.md mimo nagrania:tak, brak flow_files, itp.) zapisuj
poniżej tabeli z prefiksem `> ⚠ Info:` tak żeby user widział co poszło "po cichu".
-->

| Faza | Typ | ID | Plik artefaktu | Spec source |
|------|-----|----|----------------|-------------|
| | | | | |

## Log problemów

<!-- Format: [data] Zadanie X.X: opis problemu - rozwiązanie/status -->
<!-- Listy zmienionych plików NIE prowadzimy — git zna ją lepiej: git log --stat -->
- [2026-10-08] Start sesji: PLAN (Linux/CUDA, LLM, TTS) przeczy wymagania.md (macOS, schowek, pasek menu). Specky pusty, brak repo git i toolchainu Rust. RALPH BLOCKED, czekam na decyzję właściciela.
- [2026-10-08] Odpowiedź właściciela: tylko macOS + .dmg, Specky jedynym źródłem wymagań, git z origin gotowy. Plan przepisany, 11 wymagań zaproponowanych w Specky (wsad 01M4EK43A6GBPR63H0FV0VBCV0). Zostało: instalacja Rust, akceptacja wymagań.
- [2026-10-08] Zadanie 2.3: kryterium VA-REC-3 «pusta transkrypcja nie zmienia schowka» (crit 01M4EKHCXW1S92G9Y6AS115MW9) nie ma jeszcze testu z `specky: crit` — 2.3 daje tylko pusty wynik dla ciszy; test z schowkiem i brakiem wywołania STT powstaje w 4.1/4.3.
- [2026-10-08] Zadanie 2.3: pierwszy commit gałęzi bez dowodu kontroli (status PR czerwony) — naprawione `git commit --amend --no-edit` + push (19.4).
- [2026-10-08] Blokada rozwiązana: Rust 1.99 zainstalowany, wsad 11 wymagań zaakceptowany, kotwice w planie. Model pobierany po instalacji: dodano 5.6 i propozycję VA-MODEL-1 w Specky.
