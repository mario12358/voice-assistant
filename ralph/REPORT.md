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
- **Status**: W trakcie — Faza 9 (sygnał gotowe, szybki start, ustawienia, autostart, logi, jakość STT, wariant q5_0, wydanie w CI) dopisana 2026-10-10 z ośmiu wymagań Specky; Fazy 7 i 8 czekają tylko na testy ręczne ⛔ 7.7 i ⛔ 8.5
- **Postęp**: 67/88 pozycji ukończonych (zostały ⛔ 7.7, ⛔ 8.5 i Faza 9: 0/19)
- Specky: pracuję jako mariusz.iskra (mariusz.iskra@gmail.com), organizacja mariusz.iskra's Organization. Synchronizacja 2026-10-08 (kursor 704): 0 zmian, kolejka pusta; VA-MODEL-1 czeka na decyzję. Na prośbę właściciela VA-PLAT-1 i VA-TECH-1 oznaczone `code_ready`.
- Specky 2026-10-09 (kursor 709): kolejka pusta, VA-PLAT-2 `in_progress`.
- Otwarte PR: brak. Faza 8: PR #42–#45 zmergowane; VA-MODEL-2, VA-MODEL-3 `code_ready` (2026-10-10). Faza 7: PR #33–#38 zmergowane; VA-REC-5, VA-REC-6, VA-HIST-1 `code_ready` (2026-10-10). Faza 6: VA-PLAT-2 i VA-MODEL-1 `code_ready` (2026-10-09). Faza 5 zmergowana i otagowana (`ralph/faza-5`). `code_ready`: VA-STT-2, VA-REC-1..4, VA-UI-1, VA-UI-2. Wersja v0.2.0 utworzona (migawka Specky 01M4EVJ3FKXBKZG942NG0KYFVM); VA-STT-1 `code_ready`. Blokada kontraktu Specky rozwiązana decyzją właściciela (Specky nie obsługuje Rusta — kryteria ręcznie). VA-MODEL-1 zaakceptowane. Wersja v0.1.0 utworzona (migawka Specky 01M4EP1HG6SQR1GC25CTTSJ18P). Specky 2026-10-08 (kursor 705): kolejka pusta, VA-MODEL-1 czeka na decyzję właściciela.

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

**Przebieg po zadaniach 6.1–6.3 (2026-10-09) — ZIELONY (main 5f13dab, nie regresja fazy — czekało ⛔ 6.4):** `cargo test --workspace -- --include-ignored` 126/0 (w tym build release i bundle, montowanie .dmg, mikrofon, GPU Metal, model, schowek), clippy `-D warnings` czysto, `cargo fmt --check` czysto. Względem 119/0 po Fazie 5: +7 testów (bundle.rs 5, dmg.rs 2). Bez tagu `ralph/faza-6` do wyniku testu ręcznego 6.4.

**Regresja po Fazie 5 (2026-10-09) — ZIELONA:** na main (f76c960) `cargo test --workspace -- --include-ignored` 119/0 (z mikrofonem, GPU Metal, modelem i prawdziwym schowkiem), clippy `-D warnings` czysto, `cargo fmt --check` czysto. Względem 88/0 po Fazie 4: +31 testów. Tag `ralph/faza-5`. Zachowanie GUI (kliknięcia, skróty w innych aplikacjach, powiadomienia, wygląd ikony) poza zasięgiem testów automatycznych — scenariusze ręczne.

**Regresja po Fazie 4 (2026-10-09) — ZIELONA:** na main (96956d8) `cargo test --workspace -- --include-ignored` 88/0 (z mikrofonem, GPU Metal, modelem i prawdziwym schowkiem), clippy `-D warnings` czysto, `cargo fmt --check` czysto. Względem 70/0 po Fazie 3: +18 testów. Tag `ralph/faza-4`.

**Regresja po Fazie 3 (2026-10-09) — ZIELONA:** na main (ec17148) `cargo test --workspace -- --include-ignored` 70/0 (z testami mikrofonu, GPU Metal i prawdziwego modelu), clippy `-D warnings` czysto, `cargo fmt --check` czysto. Względem 49/0 po Fazie 2: +21 testów. Tag `ralph/faza-3`.

| Metryka | Wartość |
|---------|---------|
| Łącznie testów | 171 |
| Pass | 171 |
| Fail | 0 |
| Skip | 0 (z --include-ignored) |
| Ostatnie uruchomienie | 2026-10-10 (pełny suite na main po 8.4) |

## Historia realizacji

| Zadanie | Status | Testy | Próby | Commit | Czas |
|---------|--------|-------|-------|--------|------|
| 3.4 `va-dev transcribe` (WAV dowolny → 16 kHz mono → cisza → Whisper) + test 3.4 | ✅ PR #10 | 64/0 + 6 ignored; prawdziwa transkrypcja WAV 44,1 kHz stereo zielona lokalnie; mutacja normalizacji czerwona | 1 | ec17148 | 2026-10-09 |
| 4.1 Schowek: TextSink, ClipboardSink (reguła VA-REC-3), arboard + MemoryClipboard + test 4.1 | ✅ PR #13 | 68/0 + 7 ignored; prawdziwy schowek zielony lokalnie; mutacja reguły pustej transkrypcji czerwona | 1 | 45d1f38 | 2026-10-09 |
| 4.2 Automat stanów (StateMachine, Input/Action/Transition) + test 4.2 (tablica przejść) | ✅ PR #15 | 75/0 + 7 ignored; mutacja Start w Recording czerwona | 1 | fc2852e | 2026-10-09 |
| 4.3 Kontroler (wątek kontrolera + wątek transkrypcji, zdarzenia dla UI) + test 4.3 | ✅ PR #16 | 81/0 + 7 ignored; 3× powtórzony bez flaków; mutacja pomijania ciszy czerwona | 1 | 96956d8 | 2026-10-09 |
| 5.1 Aplikacja paska menu (tao + tray-icon, Accessory), ikona stanu rysowana w kodzie + test 5.1 | ✅ PR #18 | 86/0 + 7 ignored; aplikacja startuje lokalnie (GPU, model, pętla zdarzeń); mutacja koloru Recording czerwona | 1 | 7559763 | 2026-10-09 |
| 5.2 Kliknięcie ikony → Start/Stop (click::command_for, zdarzenia tray-icon w pętli tao) + test 5.2 | ✅ PR #19 | 91/0 + 7 ignored; mutacja czerwone→Start czerwona | 1 | 9a161ae | 2026-10-09 |
| 5.3 Menu: podmenu Mikrofon (zaznaczony używany, zapis do konfiguracji), Zakończ + test 5.3 | ✅ PR #20 | 96/0 + 7 ignored; mutacja zaznaczenia czerwona; aplikacja z menu startuje lokalnie | 1 | d3a22a4 | 2026-10-09 |
| 5.4 Skróty globalne ctrl+cmd+r / ctrl+cmd+s (global-hotkey), komunikat w menu przy konflikcie + test 5.4 | ✅ PR #21 | 100/0 + 7 ignored; rejestracja skrótów lokalnie bez błędu; mutacja zamiany Start/Stop czerwona | 1 | b7458d8 | 2026-10-09 |
| 5.5 Komunikaty: cisza cyfrowa → NoSignal + powiadomienie, brak GPU/modelu → menu + Start z powodem + test 5.5 | ✅ PR #22 | 106/0 + 7 ignored; mutacja wykrywania ciszy cyfrowej czerwona; start bez modelu lokalnie OK | 1 | 2607ff7 | 2026-10-09 |
| 5.6 Pobieranie modelu w tle (DownloadState, status i „Ponów” w menu, kontroler po pobraniu) + test 5.6 | ✅ PR #23 | 112/0 + 7 ignored; mutacja blokady Startu czerwona; start bez modelu pobiera w tle (121 MB/8 s) | 1 | f76c960 | 2026-10-09 |
| 6.1 Bundle VoiceAsystent.app (`scripts/build-app.sh`, Info.plist, ikona z `scripts/app-icon.py`, podpis ad-hoc) + test 6.1 | ✅ PR #26 | 116/0 + 8 ignored; build release + bundle 8,4 MB + start przez `open` z załadowaniem modelu na Metalu lokalnie OK; mutacja klucza NSMicrophoneUsageDescription czerwona | 1 | b188bf1 | 2026-10-09 |
| 6.2 `scripts/build-dmg.sh` (hdiutil UDZO, aplikacja + skrót do Applications, wersja z git describe) + test 6.2 | ✅ PR #27 | 118/0 + 8 ignored; obraz release 3,9 MB montuje się z aplikacją i skrótem; mutacja usunięcia skrótu czerwona | 1 | 3530198 | 2026-10-09 |
| 6.3 README.md + docs/RUNBOOK.md (wymagania, instalacja z .dmg i Gatekeeper, pierwsze uruchomienie, skróty, komendy, diagnostyka) | ✅ PR #28 | bez testów (dokumentacja); twierdzenia zweryfikowane w kodzie (limit nagrania, `.part`, SHA-256) | 1 | 5f13dab | 2026-10-09 |
| 6.4 Test manualny właściciela (instalacja z .dmg, nagranie skrótami i kliknięciem, cmd+v w kilku aplikacjach, zmiana mikrofonu) | ✅ ręcznie | właściciel 2026-10-09 wg docs/test-scenarios/v0.4.0.md: wszystkie scenariusze zgodne z oczekiwaniami, bariery odrzucone; 2.2 (macOS 13–14) nie do sprawdzenia | 1 | — | 2026-10-10 |
| 7.1 `compress_pauses` w va-audio (pauzy > 1,5 s → 0,5 s, mowa i brzegi bez zmian) + test 7.1 | ✅ PR #33 | 124/0 + 8 ignored; mutacja progu 1,5 s → 0,5 s czerwona (2 testy) | 1 | 3f44694 | 2026-10-10 |
| 7.2 Wpięcie `compress_pauses` w kontrolerze i `va-dev transcribe`, fixture `speech_pl_long_pause.wav` (2 zdania + 40 s ciszy) + test 7.2 | ✅ PR #34 | 126/0 + 9 ignored; prawdziwa transkrypcja fixture: oba zdania, 17 słów, nic między nimi; mutacja (pominięcie compress_pauses w kontrolerze) czerwona | 1 | bc0cc03 | 2026-10-10 |
| 7.3 Limit nagrania: domyślnie 600 s, `Recorder::start(…, on_limit)` z wątku audio, kontroler `LimitReached(nr)` → Stop + `ControllerEvent::LimitReached` + test 7.3 | ✅ PR #35 | 129/0 + 10 ignored; wątek audio zgłasza limit 1 s z prawdziwego mikrofonu; mutacja (ignorowanie sygnału limitu) czerwona | 1 | 5860bfb | 2026-10-10 |
| 7.4 Powiadomienie „Osiągnięto limit długości nagrania” (Problem::RecordingLimitReached, `problem_for` w pętli tao), README/RUNBOOK: limit 10 min, pauzy + test 7.4 | ✅ PR #36 | 133/0 + 10 ignored; mutacja (LimitReached → None) czerwona | 1 | 5ea081c | 2026-10-10 |
| 7.5 `va_core::history` (30 wpisów, plik JSON 0600, clear), kontroler: wpis po transkrypcji, `HistoryChanged`, `CopyHistoryEntry` przez TextSink, `ClearHistory`; `Paths::history_file` + test 7.5 | ✅ PR #37 | 143/0 + 10 ignored; mutacja limitu 30 → 31 czerwona (po poprawce testu, który porównywał ze stałą); test logów bez treści wpisu | 1 | c5271be | 2026-10-10 |
| 8.1 `ModelSource::{Store, Custom}` z `config.model_path`: własny plik bez SHA-256 i pobierania, brak pliku → `Problem::CustomModelMissing` ze ścieżką w menu; `ModelDownload::needed` pomija własną ścieżkę + test 8.1 | ✅ PR #42 | 152/0 + 10 ignored; testy startu z HOME tymczasowym (własna ścieżka istniejąca/nieistniejąca); mutacja (ignorowanie model_path) czerwona | 1 | f8a4c80 | 2026-10-10 |
| 8.2 `ModelStore::remove/size_on_disk/spec`, `ModelSpec::display_name`; `DownloadState::Missing`, zdarzenie `Removed`, `Retry` z `Missing`, `can_remove`; `ModelDownload::available` + test 8.2 | ✅ PR #43 | 156/0 + 10 ignored; mutacja (`can_remove` prawdziwe przy pobieraniu) czerwona | 1 | 17eba66 | 2026-10-10 |
| 8.3 Podmenu „Model” (model_menu.rs: linie stanu z rozmiarem w GB, własna ścieżka, flagi Usuń/Pobierz; TrayMenu::show_model; odświeżanie w pętli tao; Pokaż w Finderze przez `open -R`) + test 8.3 | ✅ PR #44 | 160/0 + 10 ignored; mutacja (Usuń aktywne przy pobieraniu) czerwona | 1 | efd85e8 | 2026-10-10 |
| 9.7 Wariant q5_0: `ModelVariant` (`model_variant` w config.toml, domyślnie full), `LARGE_V3_TURBO_Q5_0` (574 041 195 B, SHA-256 z LFS HF) i `spec_for`, `ModelSource::Store(store, wariant)`, pobieranie i usuwanie per wariant, `va-dev model-download --variant` + test 9.7 | PR #55 otwarty | 189/0 + 14 ignored; q5_0 pobrany (54 s) i rozpoznaje speech_pl; mutacja (wariant ignorowany) czerwona | 1 | — | 2026-10-10 |
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

### 2026-10-10 2026-10-10-specky-TGDZ530.md
- **Rodzaj**: rozwój
- **Priorytet**: normalny
- **Dotyczy**: ogólne (VA-UX-1; kontroler, messages.rs, app.rs, va-config)
- **Streszczenie**: Sygnał „transkrypcja gotowa”: powiadomienie z pierwszymi 60 znakami i opcjonalny dźwięk, wyłączalne w konfiguracji.
- **Zmodyfikowane zadania pending**: brak
- **Nowa faza rework**: brak
- **Nowe zadania**: 9.3, 9.10 (Faza 9)

### 2026-10-10 2026-10-10-specky-2QWNE2N.md
- **Rodzaj**: rozwój
- **Priorytet**: normalny
- **Dotyczy**: ogólne (VA-PERF-1; crates/model, startup.rs)
- **Streszczenie**: Znacznik (rozmiar, data, suma) obok modelu zamiast liczenia SHA-256 1,6 GB przy każdym starcie.
- **Zmodyfikowane zadania pending**: brak
- **Nowa faza rework**: brak
- **Nowe zadania**: 9.2, 9.10 (Faza 9)

### 2026-10-10 2026-10-10-specky-ZW632WW.md
- **Rodzaj**: rozwój
- **Priorytet**: normalny
- **Dotyczy**: ogólne (VA-SET-1; tray_menu.rs, app.rs, kontroler, va-stt, va-audio)
- **Streszczenie**: Podmenu „Ustawienia” (język, limit nagrania) z zapisem do config.toml i działaniem bez restartu.
- **Zmodyfikowane zadania pending**: brak
- **Nowa faza rework**: brak
- **Nowe zadania**: 9.5, 9.10 (Faza 9)

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
- [2026-10-10] Zadanie 9.4: `no_speech_probability()` whisper.cpp dla large-v3-turbo wynosi 0,000 dla każdego segmentu, także szumu (model uważa szum za mowę), więc sam próg no-speech nic nie odrzuca. Pomiar średniej pewności tokenów tekstu: szum 6 s → „so” 0,048; mowa w fixtures 0,90–0,997 (pojedyncze tokeny do 0,24). Dodany próg `min_segment_confidence = 0,3` na średnią segmentu. Uwaga: `cargo test -p va-stt` bez `--features metal` nie ładuje modelu (cecha włączana tylko przez binarki; w `--workspace` cechy się sumują) — testy modelu uruchamiać z `--workspace` albo `--features metal`.
- [2026-10-10] [check] PR #51 — test: wyścig w atrapie `FakeRecorder::reach_limit_of` (zadanie 7.3) — kontroler publikuje `StateChanged(Recording)` przed `Recorder::start`, więc na runnerze CI test wołał sygnał limitu, zanim atrapa go zarejestrowała (indeks poza zakresem, zatruta blokada). Lokalnie zielony 3×, w CI flaky. Naprawa: pomocnik czeka do `WAIT` na rejestrację nagrania; asercje testu bez zmian, kod produkcyjny bez zmian. Wykryte przez: test (CI).
- [2026-10-10] Zadanie 9.3: zmiana testów kontrolera `stop_puts_transcript_in_clipboard_and_reports_states_to_ui` (w sekwencji doszło `TranscriptReady` po `Delivered`) i `limit_reached_…` (filtr pomija też `TranscriptReady`) oraz testu `full_file_overrides_every_field` (nowe pola konfiguracji) — VA-UX-1, kryteria 01M4KD152NZM9VP8S1V8VX1E6R i 01M4KD152NK6764Z8AMZPSHQQN; asercje stanów i schowka bez zmian. Przy okazji: log zdarzeń w pętli tao wypisywał dotąd całe zdarzenie (`?event`) na poziomie debug, czyli także treść historii — teraz tylko rodzaj (`event_kind`), co domyka też kryterium 6 VA-HIST-1 przy debug. `app::run` dostał `AppSettings` (clippy: za dużo argumentów).
- [2026-10-10] Zadanie 9.2: znacznik jako prosty tekst `rozmiar mtime_ns sha256` (bez nowej zależności serde w va-model) w pliku `ggml-large-v3-turbo.bin.verified`; plan zakładał `.verified.json` — format nie ma znaczenia dla kryteriów. Test pominięcia sumy działa przez podmianę treści przy tym samym rozmiarze i dacie (check nadal Ready) — to jednocześnie dokumentuje świadomy kompromis VA-PERF-1. `va-dev model-download` korzysta z mechanizmu przez `ModelStore::download` → `check`. Kryterium 3 (ikona ≤ 1,5 s) zmierzone pośrednio: `--self-check` release 0,04 s przy aktualnym znaczniku; ikona w teście ręcznym 9.10.
- [2026-10-10] Zadanie 8.4: ostrzeżenie kontroli `testy/kod-bez-testu` (5 plików app) — testy w `#[cfg(test)] mod tests` (znana pułapka). Pełna ścieżka Usuń → Pobierz ponownie → nagranie bez restartu wymaga ponownego pobrania 1,6 GB i kliknięć w prawdziwym menu — test ręczny 8.5; automat potwierdzenia i automat pobierania (Missing → Retry → Downloading → Ready) pokryte testami jednostkowymi, ścieżka ModelLoaded → nowy kontroler to istniejąca ścieżka z 5.6.
- [2026-10-10] Zadanie 8.3: ostrzeżenie kontroli `testy/kod-bez-testu` (6 plików app) — testy w `#[cfg(test)] mod tests` (znana pułapka). Rozmiar modelu w menu zaokrąglany w górę do dziesiątych GB (1 624 555 275 B → „1,7 GB”); „Pokaż w Finderze” i wygląd podmenu — test ręczny 8.5.
- [2026-10-10] Zadanie 8.2: `DownloadEvent::Removed` i `DownloadState::can_remove` mają tymczasowo `#[allow(dead_code)]` — konstruowane/używane dopiero przez podmenu „Model” (8.3) i usuwanie (8.4); atrybuty do zdjęcia w tych zadaniach (clippy `-D warnings` nie przepuszcza nieużywanych symboli, a dzielenie automatu między PR-y bez tego byłoby niemożliwe).
- [2026-10-10] Zadanie 8.1: `menu_notice` zwraca teraz `Option<String>` (komunikat ze ścieżką) — test `missing_model_stays_in_menu` porównuje przez `as_deref()`, asercja bez zmian. Własna ścieżka modelu nie jest sprawdzana sumą SHA-256 (plik użytkownika, nieznana suma) — przy błędnym pliku błąd ładowania modelu zgłosi `WhisperStt::load` jak dotąd.
- [2026-10-10] Zadanie 7.6: ostrzeżenie kontroli `testy/kod-bez-testu` (app.rs, history_menu.rs, main.rs, tray_menu.rs) — testy w `#[cfg(test)] mod tests` tych samych plików (znana pułapka). Wygląd podmenu „Historia”, kliknięcie pozycji i „Wyczyść historię” w prawdziwym menu — test ręczny 7.7; przebudowa podmenu przez `remove_at(0)` w pętli jak przy mikrofonach.
- [2026-10-10] Zadanie 7.5: zmiana testów kontrolera `stop_puts_transcript_in_clipboard_and_reports_states_to_ui` i `limit_reached_stops_recording_and_transcribes_without_stop_command` — w sekwencji zdarzeń doszło `HistoryChanged` po `Delivered` (VA-HIST-1 kryterium 01M4K06AH032985T6JXJX7EE3W: wpis po każdej niepustej transkrypcji), asercje stanów i schowka bez zmian. Nowe zależności: `chrono` (czas lokalny wpisu, bez `unsafe`) i `serde_json` (plik historii) — Cargo.lock zaktualizowany.
- [2026-10-10] Zadanie 7.4: ostrzeżenie kontroli `testy/kod-bez-testu` (app.rs, main.rs, messages.rs) — testy są w `#[cfg(test)] mod tests` tych samych plików (znana pułapka), nie do naprawy przenoszeniem. Powiadomienie systemowe przy limicie niezweryfikowane wizualnie — test ręczny 7.7 (limit 20 s w config.toml).
- [2026-10-10] Zadanie 7.3: zmiana testów `partial_file_keeps_defaults_for_missing_fields` (va-config) i `config_without_file_prints_defaults` (va-dev) — oczekiwany domyślny limit 300 → 600 s. Powód: kryterium 01M4K06AY61DWRJ5R2DFNDJD6W (VA-REC-6, changes/processed/2026-10-10-specky-9VFYND0.md) ustala domyślne 600 s. Spóźniony sygnał limitu z poprzedniego nagrania jest odrzucany po numerze nagrania (test `stale_limit_signal_from_previous_recording_is_ignored`).
- [2026-10-10] Zadanie 7.2: zmiana testu `transcribe_keeps_both_sentences_around_a_long_pause_without_extra_phrases` — słowo kluczowe «biurze» → «jutro»: syntezator `say` (Zosia) wymawia „biurze” tak, że Whisper daje „Białże”; kryterium 01M4K06AR8TJXY24K05X0MNBJC (VA-REC-5, changes/processed/2026-10-10-specky-1BY4F4Z.md) wymaga obu zdań bez dodatkowych fraz, nie rozpoznania konkretnego słowa. Wynik prawdziwej transkrypcji: „Dzisiaj jest piękna pogoda, idę na spacer do parku. Jutro rano mam spotkanie w Białże o 9.00.” — 17 słów, bez halucynacji w 40 s pauzy.
- [2026-10-10] Start Fazy 7: pełny suite po przetworzeniu changes/ nie powtórzony — kod (crates/apps/scripts/Cargo) bajt w bajt ten sam, co w regresji Fazy 6 z 2026-10-10 (126/0); zmiany dotyczyły tylko ralph/, docs/ i changes/.
- [2026-10-09] Zadanie 6.1: odstępstwo od planu — skrypt nie przekazuje `--features metal`: feature `metal` jest włączony na stałe w zależności `va-stt` binarki, więc `cargo build --release -p voice-asystent` daje build z Metalem (potwierdzone logiem `ggml_metal_library_init: using embedded metal library` po starcie z bundla).
- [2026-10-09] Zadanie 6.1: ostrzeżenie kontroli `testy/skip-z-powodem` — test `release_build_creates_dist_app` ma `#[ignore]` (build release trwa minuty), uruchamiany w pełnym suicie `--include-ignored`; zielony lokalnie (40 s przy ciepłym cache). Pozostałe 4 testy bundla biegną w CI z binarki debug (`--binary`).
- [2026-10-09] Zadanie 6.1: wygląd ikony aplikacji w Finderze i nazwa aplikacji w powiadomieniach niezweryfikowane wizualnie — test ręczny 6.4. LSMinimumSystemVersion ustawione na 13.0 (brak wymagania w spec; whisper.cpp Metal i tao wymagają nowoczesnego macOS).
- [2026-10-09] Zadanie 5.1: zmiana testów startup_reports_gpu_verdict / startup_reports_missing_model — dodany argument `--self-check` (asercje bez zmian). Powód: VA-UI-1 (Specky 01M4EKHD1BQYRNWBAVRA5FJTRC) wymaga, by aplikacja trwała w pasku menu, więc bez flagi proces nie kończy się i test by wisiał.
- [2026-10-09] Zadanie 5.1: odstępstwo od planu — ikony nie jako pliki PNG @1x/@2x, tylko RGBA 36 px rysowane w kodzie (ten sam efekt na Retinie, bez binarnych zasobów, kolor sprawdzalny testem). Wygląd w pasku menu niezweryfikowany wizualnie (brak uprawnienia do nagrywania ekranu) — do testu ręcznego.
- [2026-10-09] Zadanie 5.1: start aplikacji liczy SHA-256 modelu ~3,5 s przed pokazaniem ikony — do rozważenia przy 5.6 (np. sprawdzanie w tle).
- [2026-10-09] Zadanie 4.3: AC «wejście — polecenia Start/Stop z 5.2 i 5.4» — wejście kliknięciem podpięte w 5.2 (`app.rs` → `controller.send`), skróty w 5.4.
- [2026-10-09] Zadanie 5.5: odczyt statusu zgody na mikrofon (AVCaptureDevice.authorizationStatus) wymaga `unsafe` (metoda może rzucić NSException), a workspace ma `unsafe_code = deny` — zamiast tego wykrywanie ciszy cyfrowej (same zera po Stop), którą macOS podaje bez zgody. Uwaga: wirtualne wejście (np. BlackHole) bez dźwięku też da ten komunikat. Powiadomienia przez mac-notification-sys (bez bundla pokazują się jako aplikacja Terminal — do weryfikacji po 6.1).
- [2026-10-09] Zadanie 5.5: brak miejsca na dysku w trakcie testów (256 MB wolnego) — wyczyszczone artefakty buildu (`cargo clean` dla katalogu mutacji w scratchpadzie i `--release`), wolne 8,3 GB; testy powtórzone, zielone.
- [2026-10-09] Zadanie 5.4: naciśnięcie skrótu w innej aplikacji niezweryfikowane automatycznie (symulacja klawiszy wymagałaby uprawnienia Dostępność dla terminala) — do testu ręcznego; ostrzeżenie `kod-bez-testu` jak w 5.2.
- [2026-10-09] Zadanie 5.3: lista mikrofonów odświeżana przy najechaniu na ikonę (TrayIconEvent::Enter) i prawym kliknięciu — obsługa idzie przez pętlę zdarzeń asynchronicznie, więc menu otwarte bez wcześniejszego najechania może pokazać listę sprzed chwili; do sprawdzenia ręcznie (podłączenie słuchawek). Ostrzeżenie `kod-bez-testu` jak w 5.2.
- [2026-10-09] Zadanie 5.2: ostrzeżenie kontroli `testy/kod-bez-testu` — testy Rusta są w tym samym pliku (`#[cfg(test)] mod tests` w click.rs), czego kontrola nie widzi; podpięcie w app.rs (pętla tao) bez testu automatycznego — sprawdzi test ręczny (scenariusz po Fazie 5).
- [2026-10-09] [check] PR #9 — Ralph: kontrole (lokalnie): kontrola `zaleznosci` nie wykonała się (UnicodeDecodeError) — `ralph-kontrole/zaleznosci/run.py:311-315` czyta każdy zmieniony plik jako UTF-8 przed sprawdzeniem, czy to manifest, i łapie tylko OSError; binarne WAV z fixtures ją wywracają (też przy tagu ralph/faza-2 i v0.1.0). Błąd frameworka, nie kodu — RALPH BLOCKED; 2026-10-09 właściciel naprawił kontrolę (pomija pliki niebędące manifestem) i PATH hooka (~/.cargo/bin), blokada usunięta.
