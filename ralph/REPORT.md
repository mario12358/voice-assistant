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
- **Postęp**: 29/47 pozycji ukończonych
- Specky: pracuję jako mariusz.iskra (mariusz.iskra@gmail.com), organizacja mariusz.iskra's Organization. Synchronizacja 2026-10-08 (kursor 704): 0 zmian, kolejka pusta; VA-MODEL-1 czeka na decyzję. Na prośbę właściciela VA-PLAT-1 i VA-TECH-1 oznaczone `code_ready`.
- Otwarte PR: #18 (5.1). Wersja v0.2.0 utworzona (migawka Specky 01M4EVJ3FKXBKZG942NG0KYFVM); VA-STT-1 `code_ready`. Blokada kontraktu Specky rozwiązana decyzją właściciela (Specky nie obsługuje Rusta — kryteria ręcznie). VA-MODEL-1 zaakceptowane. Wersja v0.1.0 utworzona (migawka Specky 01M4EP1HG6SQR1GC25CTTSJ18P). Specky 2026-10-08 (kursor 705): kolejka pusta, VA-MODEL-1 czeka na decyzję właściciela.

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

**Regresja po Fazie 4 (2026-10-09) — ZIELONA:** na main (96956d8) `cargo test --workspace -- --include-ignored` 88/0 (z mikrofonem, GPU Metal, modelem i prawdziwym schowkiem), clippy `-D warnings` czysto, `cargo fmt --check` czysto. Względem 70/0 po Fazie 3: +18 testów. Tag `ralph/faza-4`.

**Regresja po Fazie 3 (2026-10-09) — ZIELONA:** na main (ec17148) `cargo test --workspace -- --include-ignored` 70/0 (z testami mikrofonu, GPU Metal i prawdziwego modelu), clippy `-D warnings` czysto, `cargo fmt --check` czysto. Względem 49/0 po Fazie 2: +21 testów. Tag `ralph/faza-3`.

**Regresja po Fazie 2 (2026-10-08) — ZIELONA:** na main (1d996a2) `cargo test --workspace -- --include-ignored` 49/0 (z 3 testami mikrofonu), clippy `-D warnings` czysto, `cargo fmt --check` czysto. Względem 17/0 po Fazie 1: +32 testy. Tag `ralph/faza-2`.

**Regresja po Fazie 1 (2026-10-08) — ZIELONA:** `cargo test --workspace -- --include-ignored` 17/0 (0 ignored), clippy `-D warnings` czysto, `cargo fmt --check` czysto. Tag `ralph/faza-1`.

| Metryka | Wartość |
|---------|---------|
| Łącznie testów | 93 |
| Pass | 86 |
| Fail | 0 |
| Skip | 7 (ignored: mikrofon, GPU, model, schowek) |
| Ostatnie uruchomienie | 2026-10-08 (cargo test --workspace po 5.1) |

## Historia realizacji

| Zadanie | Status | Testy | Próby | Commit | Czas |
|---------|--------|-------|-------|--------|------|
| 1.1 Architektura i konwencje | ✅ | n/d (dokument) | 1 | dd3d2cc | 2026-10-08 |
| 1.2 Workspace Cargo + test 1.2 | ✅ | 2/0 | 1 | 70a8fe2 | 2026-10-08 |
| 1.3 Konfiguracja TOML + test 1.3 (`va-dev config`, start aplikacji) | ✅ | 14/0, mutacja numeru linii czerwona | 1 | 99d8692 | 2026-10-08 |
| 1.4 Logowanie tracing + test 1.4 | ✅ | 17/0, mutacja info→treść czerwona | 1 | 229aad4 | 2026-10-08 |
| 2.1 Lista mikrofonów, wybór z fallbackiem, `va-dev devices` + test 2.1 | ✅ | 23/0 (z ignored), mutacja fallbacku czerwona | 1 | cc5599a | 2026-10-08 |
| 2.2 Nagrywanie do bufora → 16 kHz mono (rubato Fft) + test 2.2, strażnik „bez zapisu na dysk” | ✅ | 33/0 + 2 ignored (nagranie z mikrofonu zielone lokalnie), mutacja limitu czerwona | 1 | 246b3b1 | 2026-10-08 |
| 2.3 Przycinanie ciszy (VAD RMS, ramki 20 ms) + test 2.3 na fixtures WAV | ✅ PR #1 | 41/0 + 2 ignored, mutacja progu czerwona (3 testy) | 1 | a6b7975 | 2026-10-08 |
| 2.4 `va-dev mic-test` (poziom dBFS, granice mowy, WAV z --save) + test 2.4 | ✅ PR #2 | 46/0 + 3 ignored (mic-test z mikrofonem zielony lokalnie) | 1 | 1d996a2 | 2026-10-08 |
| 3.1 Weryfikacja GPU Metal (objc2-metal) przy starcie + test 3.1 | ✅ PR #5 | 50/0 + 4 ignored (prawdziwe urządzenie Metal zielone lokalnie), mutacja wpięcia w start czerwona | 1 | a4b5a09 | 2026-10-08 |
| 3.2 Pobieranie modelu (ureq, Range, SHA-256, .part → rename), `va-dev model-download`, stan modelu przy starcie + test 3.2 | ✅ PR #8 | 58/0 + 4 ignored; mutacje sumy SHA-256 i offsetu Range czerwone; prawdziwe pobranie z HF (1,6 GB) zweryfikowane | 1 | fe42004 | 2026-10-08 |
| 3.3 WhisperStt (whisper-rs 0.16, Metal), SpeechToText + ScriptedStt, nagrania PL/EN + test 3.3 | ✅ PR #9 | 59/0 + 5 ignored; model na Metal: PL i EN rozpoznane, 5 s; mutacja use_gpu(false) czerwona | 1 | ca267d6 | 2026-10-09 |
| 3.4 `va-dev transcribe` (WAV dowolny → 16 kHz mono → cisza → Whisper) + test 3.4 | ✅ PR #10 | 64/0 + 6 ignored; prawdziwa transkrypcja WAV 44,1 kHz stereo zielona lokalnie; mutacja normalizacji czerwona | 1 | ec17148 | 2026-10-09 |
| 4.1 Schowek: TextSink, ClipboardSink (reguła VA-REC-3), arboard + MemoryClipboard + test 4.1 | ✅ PR #13 | 68/0 + 7 ignored; prawdziwy schowek zielony lokalnie; mutacja reguły pustej transkrypcji czerwona | 1 | 45d1f38 | 2026-10-09 |
| 4.2 Automat stanów (StateMachine, Input/Action/Transition) + test 4.2 (tablica przejść) | ✅ PR #15 | 75/0 + 7 ignored; mutacja Start w Recording czerwona | 1 | fc2852e | 2026-10-09 |
| 4.3 Kontroler (wątek kontrolera + wątek transkrypcji, zdarzenia dla UI) + test 4.3 | ✅ PR #16 | 81/0 + 7 ignored; 3× powtórzony bez flaków; mutacja pomijania ciszy czerwona | 1 | 96956d8 | 2026-10-09 |
| 5.1 Aplikacja paska menu (tao + tray-icon, Accessory), ikona stanu rysowana w kodzie + test 5.1 | ⏳ PR #18 | 86/0 + 7 ignored; aplikacja startuje lokalnie (GPU, model, pętla zdarzeń); mutacja koloru Recording czerwona | 1 | c7af937 (gałąź) | 2026-10-09 |

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
- [2026-10-08] Zadanie 2.4: kontrola lint-typy ostrzega „linter niedostępny (cargo)” — hook nie widzi `~/.cargo/bin` w PATH, więc commit przeszedł BEZ clippy/fmt w hooku. Clippy `-D warnings` i fmt uruchomione ręcznie — czysto. Do decyzji właściciela: PATH dla hooka albo pełna ścieżka w config.
- [2026-10-08] [check] PR #2 — Specky contract check: brak trailera Specky-Req (zadanie narzędziowe bez wymagania) — dopisany `Specky-Req: none` w commicie i w treści squasha.
- [2026-10-08] [check] PR #5 — Specky contract check: kryterium VA-STT-2 K2 «brak testu» mimo `// specky: crit` nad `#[test]` (Specky nie przeskakuje atrybutów Rusta?) — próba 1: znacznik między `#[test]` a `fn` — bez zmian (has_test=false). Podejrzenie: JUnit z nextest ma classname = nazwa crate'u, bez ścieżki pliku, więc Specky nie łączy testu z plikiem źródłowym. RALPH BLOCKED.
- [2026-10-09] Zadanie 5.1: zmiana testów startup_reports_gpu_verdict / startup_reports_missing_model — dodany argument `--self-check` (asercje bez zmian). Powód: VA-UI-1 (Specky 01M4EKHD1BQYRNWBAVRA5FJTRC) wymaga, by aplikacja trwała w pasku menu, więc bez flagi proces nie kończy się i test by wisiał.
- [2026-10-09] Zadanie 5.1: odstępstwo od planu — ikony nie jako pliki PNG @1x/@2x, tylko RGBA 36 px rysowane w kodzie (ten sam efekt na Retinie, bez binarnych zasobów, kolor sprawdzalny testem). Wygląd w pasku menu niezweryfikowany wizualnie (brak uprawnienia do nagrywania ekranu) — do testu ręcznego.
- [2026-10-09] Zadanie 5.1: start aplikacji liczy SHA-256 modelu ~3,5 s przed pokazaniem ikony — do rozważenia przy 5.6 (np. sprawdzanie w tle).
- [2026-10-09] Zadanie 4.3: AC «wejście — polecenia Start/Stop z 5.2 i 5.4» — kontroler jeszcze bez wejścia w aplikacji; podpinają go 5.2 (klik ikony) i 5.4 (skróty). Do sprawdzenia przy 5.2.
- [2026-10-09] [check] PR #9 — Ralph: kontrole (lokalnie): kontrola `zaleznosci` nie wykonała się (UnicodeDecodeError) — `ralph-kontrole/zaleznosci/run.py:311-315` czyta każdy zmieniony plik jako UTF-8 przed sprawdzeniem, czy to manifest, i łapie tylko OSError; binarne WAV z fixtures ją wywracają (też przy tagu ralph/faza-2 i v0.1.0). Błąd frameworka, nie kodu — RALPH BLOCKED; 2026-10-09 właściciel naprawił kontrolę (pomija pliki niebędące manifestem) i PATH hooka (~/.cargo/bin), blokada usunięta.
- [2026-10-08] Blokada rozwiązana: Specky czyta znaczniki tylko w .py/.js/.ts — właściciel wybrał pracę bez dowodów z CI, znaczniki zostają, kryteria odhaczane ręcznie na PR. ralph/BLOCKED.md usunięty.
- [2026-10-08] Zadanie 3.1: AC «wejście — va-dev transcribe» domknięte w 3.4 (`transcribe::run` woła `require_metal`; test transcribe_without_model_points_to_model_download).
- [2026-10-08] Tag `ralph/faza-2`: kontrola `zaleznosci` nie wykonała się (UnicodeDecodeError 0xfa — prawdopodobnie czyta binarne fixtures WAV jako tekst); tag przeszedł bez niej. Do zgłoszenia właścicielowi.
- [2026-10-08] Blokada rozwiązana: Rust 1.99 zainstalowany, wsad 11 wymagań zaakceptowany, kotwice w planie. Model pobierany po instalacji: dodano 5.6 i propozycję VA-MODEL-1 w Specky.
