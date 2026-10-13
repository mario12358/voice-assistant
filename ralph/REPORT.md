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
- **Status**: Kod Faz 7–9 zmergowany, testy ręczne ⛔ 7.7, ⛔ 8.5, ⛔ 9.10 wykonane przez właściciela 2026-10-13 — wszystko działa poza powiadomieniami systemowymi (błąd → changes/2026-10-13-powiadomienia-niewidoczne.md); VA-UI-1 zmienione w Specky (żółte kółko w trakcie transkrypcji) — wejdzie przy synchronizacji
- **Postęp**: 90/90 pozycji ukończonych; czekają: regresja Faz 7–9 na main (tagi `ralph/faza-7..9`), błąd powiadomień, zmiana VA-UI-1
- Specky: pracuję jako mariusz.iskra (mariusz.iskra@gmail.com), organizacja mariusz.iskra's Organization. Synchronizacja 2026-10-11 (kursor 737): tylko migawka v0.7.0 (informacyjnie), kolejka pusta, 21 wymagań `code_ready` + 4 `done`; brak nowych plików w changes/.
- Otwarte PR: brak. Faza 9: PR #49–#58 zmergowane; VA-OPS-1, VA-PERF-1, VA-UX-1, VA-STT-3, VA-SET-1, VA-SET-2, VA-MODEL-4, VA-CI-1 `code_ready` (2026-10-10). Faza 8: PR #42–#45 zmergowane; VA-MODEL-2, VA-MODEL-3 `code_ready` (2026-10-10). Faza 7: PR #33–#38 zmergowane; VA-REC-5, VA-REC-6, VA-HIST-1 `code_ready` (2026-10-10). Faza 6: VA-PLAT-2 i VA-MODEL-1 `code_ready` (2026-10-09). Faza 5 zmergowana i otagowana (`ralph/faza-5`). `code_ready`: VA-STT-2, VA-REC-1..4, VA-UI-1, VA-UI-2. Wersja v0.2.0 utworzona (migawka Specky 01M4EVJ3FKXBKZG942NG0KYFVM); VA-STT-1 `code_ready`. Blokada kontraktu Specky rozwiązana decyzją właściciela (Specky nie obsługuje Rusta — kryteria ręcznie). VA-MODEL-1 zaakceptowane. Wersja v0.1.0 utworzona (migawka Specky 01M4EP1HG6SQR1GC25CTTSJ18P). Specky 2026-10-08 (kursor 705): kolejka pusta, VA-MODEL-1 czeka na decyzję właściciela.

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

**Przebieg po zadaniach 8.1–8.4 (2026-10-10) — ZIELONY (main c7cd4d6, nie regresja fazy — czekają ⛔ 7.7 i ⛔ 8.5):** `cargo test --workspace -- --include-ignored` 171/0, clippy `-D warnings` czysto, `cargo fmt --check` czysto. Względem 156/0 po 7.6: +15 testów (model_path, remove/size w va-model, automat pobierania, podmenu Model, potwierdzenie). Bez tagów `ralph/faza-7` i `ralph/faza-8` do wyników testów ręcznych.

**Przebieg po zadaniach 7.1–7.6 (2026-10-10) — ZIELONY (main 6627cc3, nie regresja fazy — czeka ⛔ 7.7):** `cargo test --workspace -- --include-ignored` 156/0 (w tym prawdziwa transkrypcja fixture z 40 s pauzy, limit z prawdziwego mikrofonu, historia z plikiem, build release i .dmg), clippy `-D warnings` czysto, `cargo fmt --check` czysto. Względem 126/0 po Fazie 6: +30 testów. Bez tagu `ralph/faza-7` do wyniku testu ręcznego 7.7.

**Regresja po Fazie 6 (2026-10-10) — ZIELONA:** na main (1ae141a) `cargo test --workspace -- --include-ignored` 126/0, clippy `-D warnings` czysto, `cargo fmt --check` czysto; test ręczny 6.4 właściciela (2026-10-09) bez uwag. Bez zmian liczby testów od przebiegu po 6.3. Tag `ralph/faza-6`.

| Metryka | Wartość |
|---------|---------|
| Łącznie testów | 211 |
| Pass | 211 |
| Fail | 0 |
| Skip | 0 (z --include-ignored) |
| Ostatnie uruchomienie | 2026-10-10 (pełny suite na main po 9.11, 4e23e64) |

## Historia realizacji

| Zadanie | Status | Testy | Próby | Commit | Czas |
|---------|--------|-------|-------|--------|------|
| 7.7 Test ręczny właściciela (historia, pauzy, limit) | ✅ | ręcznie OK poza powiadomieniem przy limicie (błąd w changes/) | 1 | — | 2026-10-13 |
| 8.5 Test ręczny właściciela (podmenu Model, usuwanie, pobieranie, model_path) | ✅ | ręcznie OK | 1 | — | 2026-10-13 |
| 9.10 Test ręczny właściciela (Ustawienia, autostart, logi, q5_0, wydanie z CI v0.7.0) | ✅ | ręcznie OK poza powiadomieniem „Transkrypcja w schowku” (błąd w changes/) | 1 | — | 2026-10-13 |
| 9.8 Wybór wariantu w podmenu „Model” (pozycje Pełny 1,6 GB / q5_0 0,6 GB z zaznaczeniem, przełączenie: zapis `model_variant` → `DownloadEvent::Switch` → pobranie albo samo załadowanie → `ModelLoaded`, „Usuń nieużywany wariant”), rozmiary zaokrąglane do najbliższej dziesiątej; README/RUNBOOK fazy 9 + test 9.8 | ✅ PR #56 | 194/0 + 14 ignored; mutacja (przełączenie bez zapisu do konfiguracji) czerwona; aplikacja startuje lokalnie | 1 | e97da01 | 2026-10-10 |
| 9.7 Wariant q5_0: `ModelVariant` (`model_variant` w config.toml, domyślnie full), `LARGE_V3_TURBO_Q5_0` (574 041 195 B, SHA-256 z LFS HF) i `spec_for`, `ModelSource::Store(store, wariant)`, pobieranie i usuwanie per wariant, `va-dev model-download --variant` + test 9.7 | ✅ PR #55 | 189/0 + 14 ignored; q5_0 pobrany (54 s) i rozpoznaje speech_pl; mutacja (wariant ignorowany) czerwona | 1 | af22fbb | 2026-10-10 |
| 9.6 „Uruchamiaj przy logowaniu”: `login_item.rs` (plist LaunchAgent `io.github.mario12358.voiceasystent`, RunAtLoad, ścieżka bundla z `current_exe`, escape XML), pozycja z zaznaczeniem w podmenu Ustawienia, nieaktywna poza bundlem; `Paths::launch_agents_dir` + test 9.6 | ✅ PR #54 | 186/0 + 13 ignored; plist przechodzi `plutil -lint`; mutacja (zaznaczenie bez sprawdzania ścieżki) czerwona | 1 | 0b4a713 | 2026-10-10 |
| 9.5 Podmenu „Ustawienia” (settings_menu.rs: Język auto/pl/en, Limit 5/10/20/30 min + „inne: N s”), zapis do config.toml, `Command::SetLanguage` → wątek STT, `Command::SetRecordingLimit` → `Recorder::set_limit`; po `ModelLoaded` nowy kontroler dostaje bieżące ustawienia + test 9.5 | ✅ PR #53 | 182/0 + 13 ignored; mutacja (limit nie trafia do nagrywarki) czerwona; aplikacja z podmenu startuje lokalnie | 1 | 7663327 | 2026-10-10 |
| 9.4 `DecodingSettings`/`DECODING` (suppress blank/nst, temperatura 0 bez fallbacku, no_context, progi no-speech i pewności segmentu 0,3), filtr segmentów, `SpeechToText::set_language`; fixtures `noise_only.wav`, `speech_pl_mixed.wav` + test 9.4 | ✅ PR #52 | 176/0 + 13 ignored; prawdziwy model: szum → pusto, mieszane PL → polski, PL/EN/40 s pauzy bez regresji; mutacja progu pewności 0,3 → 0 czerwona | 1 | bf3371a | 2026-10-10 |
| 9.3 Sygnał gotowe: `ControllerEvent::TranscriptReady(TranscriptPreview)` (60 znaków, Debug bez treści) po zapisie, pola `notify_on_transcript`/`sound_on_transcript`, `signal_ready` (powiadomienie + `afplay Glass.aiff`), log zdarzeń tylko z rodzajem; `AppSettings` dla `app::run` + test 9.3 | ✅ PR #51 | 174/0 + 10 ignored; mutacja (sygnał po pustej transkrypcji) czerwona; 1 poprawka CI (wyścig w atrapie z 7.3) | 2 | 5cae27d | 2026-10-10 |
| 9.1 `logging::prune_old_logs` (7 dni, tylko `voice-asystent.log.*`) przy starcie + „Pokaż logi” w menu (`open <katalog>`) + test 9.1 | ✅ PR #49 | 164/0 + 10 ignored; mutacja progu 7 → 9 dni czerwona | 1 | fc508c3 | 2026-10-10 |
| 9.2 Znacznik `<model>.verified` (rozmiar, mtime ns, sha256) zapisywany po weryfikacji w `check()` i `download()`, pomijanie sumy przy zgodności, `remove()` kasuje znacznik + test 9.2 | ✅ PR #50 | 167/0 + 10 ignored; prawdziwy model: start --self-check 3,38 s → 0,04 s; mutacja (ignorowanie daty) czerwona | 1 | 650e715 | 2026-10-10 |
| 8.4 „Usuń model…” z potwierdzeniem w menu (RemovePrompt), usunięcie = zamknięcie kontrolera + `ModelStore::remove` + stan Missing, „Pobierz ponownie” → pobieranie → nowy kontroler; README/RUNBOOK + test 8.4 | ✅ PR #45 | 161/0 + 10 ignored; mutacja (usunięcie bez potwierdzenia) czerwona | 1 | c7cd4d6 | 2026-10-10 |
| 7.6 Podmenu „Historia” (history_menu.rs: etykiety `HH:MM · tekst…`, id → `CopyHistoryEntry`/`ClearHistory`; TrayMenu::show_history, wpięcie w pętli tao), README/RUNBOOK + test 7.6 | ✅ PR #38 | 146/0 + 10 ignored; mutacja odwrócenia kolejności czerwona; aplikacja z podmenu startuje lokalnie | 1 | 6627cc3 | 2026-10-10 |

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


### 2026-10-10 2026-10-10-specky-S3WPKBD.md
- **Rodzaj**: rozwój
- **Priorytet**: normalny
- **Dotyczy**: ogólne (VA-SET-2; login_item.rs, tray_menu.rs, app.rs)
- **Streszczenie**: „Uruchamiaj przy logowaniu” przez LaunchAgent, zaznaczenie odzwierciedla stan, nieaktywne poza bundlem.
- **Zmodyfikowane zadania pending**: brak
- **Nowa faza rework**: brak
- **Nowe zadania**: 9.6, 9.10 (Faza 9)

### 2026-10-10 2026-10-10-specky-PV3NVAC.md
- **Rodzaj**: rozwój
- **Priorytet**: normalny
- **Dotyczy**: ogólne (VA-OPS-1; logging.rs, tray_menu.rs, main.rs)
- **Streszczenie**: „Pokaż logi” w menu i usuwanie logów starszych niż 7 dni przy starcie.
- **Zmodyfikowane zadania pending**: brak
- **Nowa faza rework**: brak
- **Nowe zadania**: 9.1, 9.10 (Faza 9)

### 2026-10-10 2026-10-10-specky-F01XRR5.md
- **Rodzaj**: rozwój
- **Priorytet**: normalny
- **Dotyczy**: ogólne (VA-STT-3; crates/stt, fixtures)
- **Streszczenie**: Parametry Whispera przeciw halucynacjom i zmiana języka w działającym modelu; szum daje pusty wynik.
- **Zmodyfikowane zadania pending**: brak
- **Nowa faza rework**: brak
- **Nowe zadania**: 9.4, 9.10 (Faza 9)

### 2026-10-10 2026-10-10-specky-1JBKCWR.md
- **Rodzaj**: rozwój
- **Priorytet**: normalny
- **Dotyczy**: ogólne (VA-MODEL-4; crates/model, va-config, startup.rs, model_menu.rs, app.rs)
- **Streszczenie**: Wariant skwantyzowany q5_0 (~0,6 GB) do wyboru w podmenu „Model” z pobieraniem i przeładowaniem bez restartu; VA-STT-1 bez zmian.
- **Zmodyfikowane zadania pending**: brak
- **Nowa faza rework**: brak
- **Nowe zadania**: 9.7, 9.8, 9.10 (Faza 9)

### 2026-10-10 2026-10-10-specky-47S54D5.md
- **Rodzaj**: rozwój
- **Priorytet**: normalny
- **Dotyczy**: ogólne (VA-CI-1; .github/workflows/release.yml, scripts/)
- **Streszczenie**: Tag v* buduje .dmg w CI i publikuje wydanie GitHub z opisem z tagu; obraz < 20 MB bez modelu.
- **Zmodyfikowane zadania pending**: brak
- **Nowa faza rework**: brak
- **Nowe zadania**: 9.9, 9.10 (Faza 9)

### 2026-10-10 2026-10-10-specky-NFWQ1DA.md
- **Rodzaj**: rozwój
- **Priorytet**: normalny
- **Dotyczy**: ogólne (nowe wymaganie Specky VA-MODEL-2; tray_menu.rs, download.rs, startup.rs, app.rs, crates/model)
- **Streszczenie**: Podmenu „Model” z nazwą, rozmiarem i stanem modelu, „Pokaż w Finderze”, „Usuń model” z potwierdzeniem w menu i „Pobierz ponownie” bez restartu aplikacji; wariant bez przełączania modeli (VA-STT-1 bez zmian).
- **Zmodyfikowane zadania pending**: brak
- **Nowa faza rework**: brak
- **Nowe zadania**: 8.2, 8.3, 8.4, 8.5 (Faza 8)

### 2026-10-10 2026-10-10-specky-XJ4CRAZ.md
- **Rodzaj**: błąd
- **Źródło błędu**: zadanie 5.1 (startup sprawdza model tylko w stałym katalogu; pole `model_path` z 1.3)
- **Wykryte przez**: ralph
- **Priorytet**: normalny
- **Dotyczy**: ogólne (nowe wymaganie Specky VA-MODEL-3; startup.rs, README)
- **Streszczenie**: Pole `model_path` z config.toml, opisane w README, honoruje tylko `va-dev transcribe`; aplikacja ma ładować wskazany plik bez pobierania, a przy braku pliku pokazać „Brak modelu” ze ścieżką.
- **Zmodyfikowane zadania pending**: brak
- **Nowa faza rework**: brak
- **Nowe zadania**: 8.1 (Faza 8; 8.3 i 8.5 współdzielone z VA-MODEL-2)

### 2026-10-10 2026-10-10-specky-CY0SQA5.md
- **Rodzaj**: rozwój
- **Priorytet**: normalny
- **Dotyczy**: ogólne (nowe wymaganie Specky VA-HIST-1, sekcja „Nagrywanie i schowek”)
- **Streszczenie**: Historia wypowiedzi: każda niepusta transkrypcja trafia do trwałej listy (30 wpisów, plik w katalogu danych aplikacji), podmenu „Historia” pozwala skopiować wcześniejszy tekst do schowka i wyczyścić listę; decyzja właściciela 2026-10-10.
- **Zmodyfikowane zadania pending**: brak
- **Nowa faza rework**: brak
- **Nowe zadania**: 7.5, 7.6, 7.7 (Faza 7)

### 2026-10-10 2026-10-10-specky-1BY4F4Z.md
- **Rodzaj**: rozwój
- **Priorytet**: normalny
- **Dotyczy**: ogólne (nowe wymaganie Specky VA-REC-5; crates/audio/src/silence.rs, kontroler)
- **Streszczenie**: Pauzy wewnątrz nagrania dłuższe niż 1,5 s są skracane do 0,5 s przed transkrypcją, żeby Whisper nie halucynował w oknach bez mowy; brzegi jak dotąd.
- **Zmodyfikowane zadania pending**: brak
- **Nowa faza rework**: brak
- **Nowe zadania**: 7.1, 7.2 (Faza 7)

### 2026-10-10 2026-10-10-specky-9VFYND0.md
- **Rodzaj**: rozwój
- **Priorytet**: normalny
- **Dotyczy**: ogólne (nowe wymaganie Specky VA-REC-6; recorder, va-config, kontroler, messages.rs)
- **Streszczenie**: Limit nagrania domyślnie 600 s; po jego osiągnięciu nagrywarka zgłasza limit, kontroler kończy nagranie jak po ctrl+cmd+s (transkrypcja do schowka), a użytkownik dostaje powiadomienie — dziś nadwyżka ginęła po cichu.
- **Zmodyfikowane zadania pending**: brak
- **Nowa faza rework**: brak
- **Nowe zadania**: 7.3, 7.4 (Faza 7)

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
- [2026-10-13] Zadania 7.7/9.10: powiadomienia systemowe (gotowa transkrypcja, limit) nie pojawiają się wcale, także w Centrum powiadomień, bez ostrzeżenia w logach — `messages.rs` (5.5) wysyła przez mac-notification-sys bez `set_application`, czyli jako com.apple.Finder; macOS 26.6 je gubi. Zgłoszone jako błąd: changes/2026-10-13-powiadomienia-niewidoczne.md. Testy automatyczne tego nie widziały (sprawdzają mapowanie zdarzenie → powiadomienie, nie dostarczenie).
- [2026-10-10] Zadanie 8.2: `DownloadEvent::Removed` i `DownloadState::can_remove` mają tymczasowo `#[allow(dead_code)]` — konstruowane/używane dopiero przez podmenu „Model” (8.3) i usuwanie (8.4); atrybuty do zdjęcia w tych zadaniach (clippy `-D warnings` nie przepuszcza nieużywanych symboli, a dzielenie automatu między PR-y bez tego byłoby niemożliwe).
- [2026-10-10] Zadanie 8.1: `menu_notice` zwraca teraz `Option<String>` (komunikat ze ścieżką) — test `missing_model_stays_in_menu` porównuje przez `as_deref()`, asercja bez zmian. Własna ścieżka modelu nie jest sprawdzana sumą SHA-256 (plik użytkownika, nieznana suma) — przy błędnym pliku błąd ładowania modelu zgłosi `WhisperStt::load` jak dotąd.
- [2026-10-10] Zadanie 7.6: ostrzeżenie kontroli `testy/kod-bez-testu` (app.rs, history_menu.rs, main.rs, tray_menu.rs) — testy w `#[cfg(test)] mod tests` tych samych plików (znana pułapka). Wygląd podmenu „Historia”, kliknięcie pozycji i „Wyczyść historię” w prawdziwym menu — test ręczny 7.7; przebudowa podmenu przez `remove_at(0)` w pętli jak przy mikrofonach.
- [2026-10-10] Zadanie 7.5: zmiana testów kontrolera `stop_puts_transcript_in_clipboard_and_reports_states_to_ui` i `limit_reached_stops_recording_and_transcribes_without_stop_command` — w sekwencji zdarzeń doszło `HistoryChanged` po `Delivered` (VA-HIST-1 kryterium 01M4K06AH032985T6JXJX7EE3W: wpis po każdej niepustej transkrypcji), asercje stanów i schowka bez zmian. Nowe zależności: `chrono` (czas lokalny wpisu, bez `unsafe`) i `serde_json` (plik historii) — Cargo.lock zaktualizowany.
- [2026-10-10] Zadanie 7.4: ostrzeżenie kontroli `testy/kod-bez-testu` (app.rs, main.rs, messages.rs) — testy są w `#[cfg(test)] mod tests` tych samych plików (znana pułapka), nie do naprawy przenoszeniem. Powiadomienie systemowe przy limicie niezweryfikowane wizualnie — test ręczny 7.7 (limit 20 s w config.toml).
- [2026-10-10] Zadanie 7.3: zmiana testów `partial_file_keeps_defaults_for_missing_fields` (va-config) i `config_without_file_prints_defaults` (va-dev) — oczekiwany domyślny limit 300 → 600 s. Powód: kryterium 01M4K06AY61DWRJ5R2DFNDJD6W (VA-REC-6, changes/processed/2026-10-10-specky-9VFYND0.md) ustala domyślne 600 s. Spóźniony sygnał limitu z poprzedniego nagrania jest odrzucany po numerze nagrania (test `stale_limit_signal_from_previous_recording_is_ignored`).
- [2026-10-10] Zadanie 7.2: zmiana testu `transcribe_keeps_both_sentences_around_a_long_pause_without_extra_phrases` — słowo kluczowe «biurze» → «jutro»: syntezator `say` (Zosia) wymawia „biurze” tak, że Whisper daje „Białże”; kryterium 01M4K06AR8TJXY24K05X0MNBJC (VA-REC-5, changes/processed/2026-10-10-specky-1BY4F4Z.md) wymaga obu zdań bez dodatkowych fraz, nie rozpoznania konkretnego słowa. Wynik prawdziwej transkrypcji: „Dzisiaj jest piękna pogoda, idę na spacer do parku. Jutro rano mam spotkanie w Białże o 9.00.” — 17 słów, bez halucynacji w 40 s pauzy.
- [2026-10-10] Start Fazy 7: pełny suite po przetworzeniu changes/ nie powtórzony — kod (crates/apps/scripts/Cargo) bajt w bajt ten sam, co w regresji Fazy 6 z 2026-10-10 (126/0); zmiany dotyczyły tylko ralph/, docs/ i changes/.
- [2026-10-10] Zadanie 9.11: pełny suite po 9.9 dał 209/1 — `model_loads_once_on_metal_and_transcribes_polish_and_english` (VA-STT-1) padał 4/4 przy całym pliku `whisper_model.rs`, sam przechodził. Przyczyna: subskrybent `tracing` przypięty do wątku testu, a logi whisper.cpp idą przez globalny hook; równoległe testy modelu dodane w 9.4 i 9.7 gubiły linie o Metalu w buforze (asercje w liniach 85 i 93). Kod aplikacji bez zmian; test przeniesiony do osobnego pliku (osobny proces), asercje bez zmian. Regresja izolacji testów — powstała w 9.4, niewidoczna, bo przebiegi zadań uruchamiały tylko testy zadania. Ostrzeżenie `testy/mniej-asercji` (whisper_model.rs 18 → 10) — asercje nie usunięte, tylko przeniesione do whisper_metal.rs razem z testem; `skip-z-powodem` jak wcześniej (model i GPU).
- [2026-10-10] [check] PR #58 — CI `test`: `release_check_accepts_small_image_without_model` (z 9.9) padł z kodem 1 bez komunikatu — `hdiutil attach -quiet` przy `set -e` kończył `check-dmg.sh` po cichu; pierwszy push tej samej gałęzi był zielony, lokalnie 1 porażka na 3 przebiegi dmg.rs. Po usunięciu `-quiet` przyczyna potwierdzona lokalnie: `hdiutil: attach failed - Zasoby czasowo niedostępne` przy kilku testach obrazów naraz (same ponawianie 3× nie wystarczyło, 1 na 5 przebiegów czerwony). Poprawka: testy obrazów szeregowane (mutex w dmg.rs dla `cargo test`, grupa nextest `obrazy-dysku` z max-threads 1 w `.config/nextest.toml` dla CI), plus 3 próby z błędem `hdiutil` na stderr w skryptach (przydatne też w workflow wydania); nowy test obrazu nie do zamontowania. Po poprawce szybki zestaw 6× zielony.
