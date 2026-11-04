# telesma/state/REPORT.md - Raport realizacji projektu

<!-- This file goes IN FULL into the startup prompt of every session. Two kinds of sections:  -->
<!--   • current state (Summary) — OVERWRITE it, it is meant to be one up-to-date description; -->
<!--   • chronicle (Test status, Task history, Change history, Problem log,                    -->
<!--     Visual artifacts) — APPEND one entry; older ones rotate to telesma/state/REPORT_ARCHIVE.md   -->
<!--     at startup (Test status: the 3 latest runs).                                          -->
<!-- Do not create your own `## ` sections for session summaries — EVERY `## ` section outside -->
<!-- this template is ad-hoc and only the 3 newest of them are kept.                           -->
<!-- Rotation moves WHOLE entries, multi-line ones too. An entry must have a recognisable      -->
<!-- start (a table row, `### …`, `- [date] …`, `- **Name …**`), and a paragraph glued        -->
<!-- under an entry counts as its continuation and goes to the archive together with it.      -->

## Summary

- **Start**: 2026-10-08
- **Status**: CZEKA NA CZŁOWIEKA (2026-11-04) — Faza 10 (tryb nagrywania spotkania, VA-CONF-1..4) zaimplementowana i zmergowana (PR #70–#77), wszystkie cztery wymagania `code_ready`. Zostają testy ręczne właściciela: ⛔ 10.8 (spotkanie w Meet, zgoda TCC, 60 min) i ⛔ 5.Z1.2 (żółte kółko); po nich regresja na main i tagi `telesma/phase-10`, `telesma/phase-5.Z1`.
- **Progress**: 106/108 pozycji ukończonych; czekają: ⛔ 5.Z1.2, ⛔ 10.8
- Specky: pracuję jako mariusz.iskra (mariusz.iskra@gmail.com), organizacja mariusz.iskra's Organization, klient agent_interactive. Synchronizacja 2026-11-04 (kursor 950): VA-CONF-1..4 zaakceptowane na prośbę właściciela i `code_ready` (2026-11-04); 25 `code_ready` + 4 `done`.
- Otwarte PR: brak (poza aktualizacją stanu). Faza 9: PR #49–#58 zmergowane; VA-OPS-1, VA-PERF-1, VA-UX-1, VA-STT-3, VA-SET-1, VA-SET-2, VA-MODEL-4, VA-CI-1 `code_ready` (2026-10-10). Faza 8: PR #42–#45 zmergowane; VA-MODEL-2, VA-MODEL-3 `code_ready` (2026-10-10). Faza 7: PR #33–#38 zmergowane; VA-REC-5, VA-REC-6, VA-HIST-1 `code_ready` (2026-10-10). Faza 6: VA-PLAT-2 i VA-MODEL-1 `code_ready` (2026-10-09). Faza 5 zmergowana i otagowana (`telesma/phase-5`). `code_ready`: VA-STT-2, VA-REC-1..4, VA-UI-1, VA-UI-2. Wersja v0.2.0 utworzona (migawka Specky 01M4EVJ3FKXBKZG942NG0KYFVM); VA-STT-1 `code_ready`. Blokada kontraktu Specky rozwiązana decyzją właściciela (Specky nie obsługuje Rusta — kryteria ręcznie). VA-MODEL-1 zaakceptowane. Wersja v0.1.0 utworzona (migawka Specky 01M4EP1HG6SQR1GC25CTTSJ18P). Specky 2026-10-08 (kursor 705): kolejka pusta, VA-MODEL-1 czeka na decyzję właściciela.

## Test status

<!--
The table is the current state — REPLACE the numbers, do not add a second table.
Above the table, one paragraph per full-suite run, opened with bold text carrying the date:

**Regression after Phase 12 (2026-08-20) — GREEN:** backend `pytest -q` 340/0, frontend 210/0.
Compared with 328/205 after Phase 11: +12 backend tests. Tag `telesma/phase-12`.

The date in the first line is MANDATORY — rotation uses it to tell which runs to keep
(the 3 latest) and which to move to telesma/state/REPORT_ARCHIVE.md. A paragraph without a date
is treated as a continuation of the run above it. A block without a date stays in the file
forever and grows in the prompt of every session.
-->

**Preliminary run after Phase 10 (2026-11-04) — GREEN (branch zadanie-10.7-menu-spotkania, kod = main d41e56f; nie regresja fazy — czeka ⛔ 10.8):** `cargo test --workspace -- --include-ignored` 260/0, clippy `-D warnings` czysto, `cargo fmt --check` czysto. Względem 217/0: +43 testy Fazy 10 (w tym prawdziwy process tap, tor spotkania z Whisperem i scalanie ścieżek na modelu).

**Przebieg przed wydaniem v0.8.0 (2026-10-13) — GREEN (main f504f30, nie regresja fazy — czeka ⛔ 5.Z1.2):** `cargo test --workspace -- --include-ignored` 217/0, clippy `-D warnings` czysto, `cargo fmt --check` czysto. Względem 211/0: +6 testów 5.Z1.1. Wersja v0.8.0 na prośbę właściciela.

**Regression after Phases 7, 8 i 9 (2026-10-13) — GREEN:** na main (3eaedc8) `cargo test --workspace -- --include-ignored` 211/0, clippy `-D warnings` czysto, `cargo fmt --check` czysto; testy ręczne ⛔ 7.7, 8.5, 9.10 właściciela OK (2026-10-13). Bez zmian liczby testów od przebiegu po 9.11. Tagi `telesma/phase-7`, `telesma/phase-8`, `telesma/phase-9`.

**Regression after Phase 6 (2026-10-10) — GREEN:** na main (1ae141a) `cargo test --workspace -- --include-ignored` 126/0, clippy `-D warnings` czysto, `cargo fmt --check` czysto; test ręczny 6.4 właściciela (2026-10-09) bez uwag. Bez zmian liczby testów od przebiegu po 6.3. Tag `telesma/phase-6`.

| Metric | Value |
|---------|---------|
| Total tests | 260 |
| Pass | 260 |
| Fail | 0 |
| Skip | 0 (z --include-ignored) |
| Last run | 2026-11-04 (pełny suite przed testem ręcznym 10.8, kod = main d41e56f) |

## Task history

| Task | Status | Tests | Tries | Commit | Time |
|---------|--------|-------|-------|--------|------|
| 10.7 aplikacja: „Nagrywaj spotkanie” (nieaktywne < macOS 14.2), ctrl+cmd+m, podpowiedź „Nagrywanie spotkania”, powiadomienia (odmowa zgody, stary system, limit spotkania), `NSAudioCaptureUsageDescription`, sprzątanie spool przy starcie, urządzenie tapu poza listą mikrofonów; poprawka częstotliwości tapu (zegar urządzenia) i dołączanie wątku transkrypcji + test 10.7 i test całego toru | ✅ PR #77 | 260/0 z --include-ignored; prawdziwy tor (mikrofon + tap + Whisper): afplay speech_en → „Rozmówcy: … park”; mutacje (filtr mikrofonów, blokada < 14.2) czerwone | 1 | d41e56f | 2026-11-04 |
| 10.6 `Command::StartMeeting`/`Input::StartMeeting`, zgoda przed zmianą stanu (`MeetingDenied`), `MeetingRecording` (CpalStreamRecorder + tap → normalizacja → 2 spool, limit `max_meeting_secs`), transkrypcja obu ścieżek + `merge_tracks` w wątku roboczym; `MeetingParts` ze startu aplikacji + test 10.6 | ✅ PR #76 | 235/0; mutacje (pominięta zgoda, brak obcięcia do limitu) czerwone | 1 | e7afc09 | 2026-11-04 |
| 10.5 crate `va-system-audio`: Core Audio process tap (globalny, prywatny, bez wyciszania) w prywatnym urządzeniu zagregowanym, zgoda TCC, macOS ≥ 14.2, `SystemAudioSource` + atrapa + test 10.5 | ✅ PR #75 | 227/0; prawdziwy tap: afplay speech_en.wav przechwycony, `system_profiler` bez nowego urządzenia; mutacja (urządzenie niepubliczne) czerwona | 1 | 262370c | 2026-11-04 |
| 10.3 `va-core::meeting`: `transcribe_track` (cisza bez modelu, znaczniki na osi ścieżki) + `merge_tracks` („Ja:”/„Rozmówcy:” przy zmianie mówcy) + test 10.3 | ✅ PR #74 | 224/0; prawdziwy model: speech_pl 0 s + speech_en 10 s → „Ja:” potem „Rozmówcy:” i odwrotnie; mutacje (sortowanie bez czasu, etykieta przy każdym segmencie) czerwone | 1 | 6925e9a | 2026-11-04 |
| 10.4 `va-audio::StreamNormalizer` + `va-core::spool` (SpoolWriter 0600 `meeting-*.pcm`, SpooledAudio usuwany przy zniszczeniu, `remove_leftovers`) + test 10.4 | ✅ PR #73 | 213/0; mutacje (0600 → 0644, brak usuwania po odczycie) czerwone | 1 | 8b40d57 | 2026-11-04 |
| 10.2 `Transcript.segments` (ms, po filtrach VA-STT-3) + `compress_pauses_mapped`/`TimeMap` (skrócona oś → oryginalna, `shifted`) + test 10.2 | ✅ PR #72 | 207/0; prawdziwy model: segmenty speech_pl rosnące w granicach nagrania; mutacja przeliczenia setnych → ms czerwona | 1 | 753f7d1 | 2026-11-04 |
| 10.1 `max_meeting_secs` (7200) i `meeting_speaker_labels` (true) w `va-config`, README + test 10.1 | ✅ PR #71 | 206/0; mutacja domyślnego limitu 7200 → 600 czerwona | 1 | 067a75d | 2026-11-04 |
| 5.Z1.1 `Dot::Yellow` w Transcribing (systemowy żółty 255/204/0), szare po zapisie do schowka / ciszy / błędzie; testy kolejności zdarzeń i Startu w trakcie transkrypcji (atrapa STT z bramką) + test 5.Z1.1 | ✅ PR #67 | 203/0 + 14 ignored; CI zielone; 3 mutacje czerwone | 1 | 206955a | 2026-10-13 |
| 7.7 Test ręczny właściciela (historia, pauzy, limit) | ✅ | ręcznie OK (powiadomienie widoczne po odblokowaniu w macOS) | 1 | — | 2026-10-13 |
| 8.5 Test ręczny właściciela (podmenu Model, usuwanie, pobieranie, model_path) | ✅ | ręcznie OK | 1 | — | 2026-10-13 |
| 9.10 Test ręczny właściciela (Ustawienia, autostart, logi, q5_0, wydanie z CI v0.7.0) | ✅ | ręcznie OK (powiadomienie widoczne po odblokowaniu w macOS) | 1 | — | 2026-10-13 |
| 9.4 `DecodingSettings`/`DECODING` (suppress blank/nst, temperatura 0 bez fallbacku, no_context, progi no-speech i pewności segmentu 0,3), filtr segmentów, `SpeechToText::set_language`; fixtures `noise_only.wav`, `speech_pl_mixed.wav` + test 9.4 | ✅ PR #52 | 176/0 + 13 ignored; prawdziwy model: szum → pusto, mieszane PL → polski, PL/EN/40 s pauzy bez regresji; mutacja progu pewności 0,3 → 0 czerwona | 1 | bf3371a | 2026-10-10 |
| 9.3 Sygnał gotowe: `ControllerEvent::TranscriptReady(TranscriptPreview)` (60 znaków, Debug bez treści) po zapisie, pola `notify_on_transcript`/`sound_on_transcript`, `signal_ready` (powiadomienie + `afplay Glass.aiff`), log zdarzeń tylko z rodzajem; `AppSettings` dla `app::run` + test 9.3 | ✅ PR #51 | 174/0 + 10 ignored; mutacja (sygnał po pustej transkrypcji) czerwona; 1 poprawka CI (wyścig w atrapie z 7.3) | 2 | 5cae27d | 2026-10-10 |
| 9.1 `logging::prune_old_logs` (7 dni, tylko `voice-asystent.log.*`) przy starcie + „Pokaż logi” w menu (`open <katalog>`) + test 9.1 | ✅ PR #49 | 164/0 + 10 ignored; mutacja progu 7 → 9 dni czerwona | 1 | fc508c3 | 2026-10-10 |
| 9.2 Znacznik `<model>.verified` (rozmiar, mtime ns, sha256) zapisywany po weryfikacji w `check()` i `download()`, pomijanie sumy przy zgodności, `remove()` kasuje znacznik + test 9.2 | ✅ PR #50 | 167/0 + 10 ignored; prawdziwy model: start --self-check 3,38 s → 0,04 s; mutacja (ignorowanie daty) czerwona | 1 | 650e715 | 2026-10-10 |
| 8.4 „Usuń model…” z potwierdzeniem w menu (RemovePrompt), usunięcie = zamknięcie kontrolera + `ModelStore::remove` + stan Missing, „Pobierz ponownie” → pobieranie → nowy kontroler; README/RUNBOOK + test 8.4 | ✅ PR #45 | 161/0 + 10 ignored; mutacja (usunięcie bez potwierdzenia) czerwona | 1 | c7cd4d6 | 2026-10-10 |
| 7.6 Podmenu „Historia” (history_menu.rs: etykiety `HH:MM · tekst…`, id → `CopyHistoryEntry`/`ClearHistory`; TrayMenu::show_history, wpięcie w pętli tao), README/RUNBOOK + test 7.6 | ✅ PR #38 | 146/0 + 10 ignored; mutacja odwrócenia kolejności czerwona; aplikacja z podmenu startuje lokalnie | 1 | 6627cc3 | 2026-10-10 |

## Change history

<!--
Entries added by Telesma after processing a file from telesma/changes/.
Entry format (one ### per change):

### YYYY-MM-DD <file-name>.md
- **Kind**: bug | correction | feature
- **Bug source**: [bugs only: task X.Y that built it | unknown]
- **Detected by**: [bugs only: test | scenario | review | telesma | owner | production]
- **Priority**: critical | normal
- **Affects**: [the affects field from the frontmatter, or "general"]
- **Summary**: [1-2 sentences on what changes and why]
- **Modified pending tasks**: [list of numbers, e.g. 4.2, 5.1] or "none"
- **New rework phase**: [e.g. Phase 6.Z1 with 3 tasks] or "none"
- **New tasks**: [list of numbers, e.g. 7.4, 7.5] or "none"

The change's source file lives in telesma/changes/processed/<file-name>.md (audit trail in git).
-->


### 2026-11-04 2026-11-04-specky-YMXASGG.md
- **Kind**: feature
- **Priority**: normal
- **Affects**: general (nowe wymaganie Specky VA-CONF-1; tray_menu.rs, hotkeys.rs, indicator.rs, kontroler)
- **Summary**: Tryb nagrywania spotkania: „Nagrywaj spotkanie” w menu i ctrl+cmd+m nagrywają mikrofon i dźwięk systemowy naraz; reszta jak zwykłe nagranie.
- **Modified pending tasks**: none
- **New rework phase**: none
- **New tasks**: 10.6, 10.7, 10.8 (Faza 10)

### 2026-11-04 2026-11-04-specky-35SNPEN.md
- **Kind**: feature
- **Priority**: normal
- **Affects**: general (nowe wymaganie Specky VA-CONF-2; nowy crate va-system-audio, Info.plist)
- **Summary**: Dźwięk systemowy przez Core Audio process tap (macOS 14.2+) ze zgodą TCC; implementacja w Ruście z `unsafe` tylko w va-system-audio (decyzja właściciela 2026-11-04).
- **Modified pending tasks**: none
- **New rework phase**: none
- **New tasks**: 10.5, 10.6, 10.7, 10.8 (Faza 10)

### 2026-11-04 2026-11-04-specky-REDTSAM.md
- **Kind**: feature
- **Priority**: normal
- **Affects**: general (nowe wymaganie Specky VA-CONF-3; va-config, va-audio, kontroler)
- **Summary**: Osobny limit spotkania 7200 s i zapis dźwięku na dysk w trakcie nagrania zamiast w RAM, plik usuwany po transkrypcji.
- **Modified pending tasks**: none
- **New rework phase**: none
- **New tasks**: 10.1, 10.4, 10.6 (Faza 10)

### 2026-11-04 2026-11-04-specky-D6FG0MQ.md
- **Kind**: feature
- **Priority**: normal
- **Affects**: general (nowe wymaganie Specky VA-CONF-4; va-stt, va-audio, va-core)
- **Summary**: Dwie ścieżki spotkania transkrybowane osobno i scalane po czasie z etykietami „Ja:”/„Rozmówcy:”, wyłączalnymi w config.toml.
- **Modified pending tasks**: none
- **New rework phase**: none
- **New tasks**: 10.1, 10.2, 10.3 (Faza 10)


### 2026-10-13 2026-10-13-specky-5FJTRC.md
- **Kind**: correction
- **Priority**: normal
- **Affects**: zadanie 5.1 (indicator.rs), VA-UI-1 v1 → v2
- **Summary**: W trakcie transkrypcji ikona ma być żółtym kółkiem (dotąd szara); szare po żółtym oznacza tekst w schowku, także po ciszy i błędzie; klik/ctrl+cmd+r przy żółtym nie startują nagrania.
- **Modified pending tasks**: none
- **New rework phase**: Phase 5.Z1 z 2 zadaniami (5.Z1.1 + test, ⛔ 5.Z1.2)
- **New tasks**: none


### 2026-10-10 2026-10-10-specky-PV3NVAC.md
- **Kind**: feature
- **Priority**: normal
- **Affects**: general (VA-OPS-1; logging.rs, tray_menu.rs, main.rs)
- **Summary**: „Pokaż logi” w menu i usuwanie logów starszych niż 7 dni przy starcie.
- **Modified pending tasks**: none
- **New rework phase**: none
- **New tasks**: 9.1, 9.10 (Faza 9)

### 2026-10-10 2026-10-10-specky-F01XRR5.md
- **Kind**: feature
- **Priority**: normal
- **Affects**: general (VA-STT-3; crates/stt, fixtures)
- **Summary**: Parametry Whispera przeciw halucynacjom i zmiana języka w działającym modelu; szum daje pusty wynik.
- **Modified pending tasks**: none
- **New rework phase**: none
- **New tasks**: 9.4, 9.10 (Faza 9)

### 2026-10-10 2026-10-10-specky-1JBKCWR.md
- **Kind**: feature
- **Priority**: normal
- **Affects**: general (VA-MODEL-4; crates/model, va-config, startup.rs, model_menu.rs, app.rs)
- **Summary**: Wariant skwantyzowany q5_0 (~0,6 GB) do wyboru w podmenu „Model” z pobieraniem i przeładowaniem bez restartu; VA-STT-1 bez zmian.
- **Modified pending tasks**: none
- **New rework phase**: none
- **New tasks**: 9.7, 9.8, 9.10 (Faza 9)

### 2026-10-10 2026-10-10-specky-47S54D5.md
- **Kind**: feature
- **Priority**: normal
- **Affects**: general (VA-CI-1; .github/workflows/release.yml, scripts/)
- **Summary**: Tag v* buduje .dmg w CI i publikuje wydanie GitHub z opisem z tagu; obraz < 20 MB bez modelu.
- **Modified pending tasks**: none
- **New rework phase**: none
- **New tasks**: 9.9, 9.10 (Faza 9)

### 2026-10-10 2026-10-10-specky-NFWQ1DA.md
- **Kind**: feature
- **Priority**: normal
- **Affects**: general (nowe wymaganie Specky VA-MODEL-2; tray_menu.rs, download.rs, startup.rs, app.rs, crates/model)
- **Summary**: Podmenu „Model” z nazwą, rozmiarem i stanem modelu, „Pokaż w Finderze”, „Usuń model” z potwierdzeniem w menu i „Pobierz ponownie” bez restartu aplikacji; wariant bez przełączania modeli (VA-STT-1 bez zmian).
- **Modified pending tasks**: none
- **New rework phase**: none
- **New tasks**: 8.2, 8.3, 8.4, 8.5 (Faza 8)

### 2026-10-10 2026-10-10-specky-XJ4CRAZ.md
- **Kind**: bug
- **Bug source**: task 5.1 (startup sprawdza model tylko w stałym katalogu; pole `model_path` z 1.3)
- **Detected by**: telesma
- **Priority**: normal
- **Affects**: general (nowe wymaganie Specky VA-MODEL-3; startup.rs, README)
- **Summary**: Pole `model_path` z config.toml, opisane w README, honoruje tylko `va-dev transcribe`; aplikacja ma ładować wskazany plik bez pobierania, a przy braku pliku pokazać „Brak modelu” ze ścieżką.
- **Modified pending tasks**: none
- **New rework phase**: none
- **New tasks**: 8.1 (Faza 8; 8.3 i 8.5 współdzielone z VA-MODEL-2)

### 2026-10-10 2026-10-10-specky-CY0SQA5.md
- **Kind**: feature
- **Priority**: normal
- **Affects**: general (nowe wymaganie Specky VA-HIST-1, sekcja „Nagrywanie i schowek”)
- **Summary**: Historia wypowiedzi: każda niepusta transkrypcja trafia do trwałej listy (30 wpisów, plik w katalogu danych aplikacji), podmenu „Historia” pozwala skopiować wcześniejszy tekst do schowka i wyczyścić listę; decyzja właściciela 2026-10-10.
- **Modified pending tasks**: none
- **New rework phase**: none
- **New tasks**: 7.5, 7.6, 7.7 (Faza 7)

### 2026-10-10 2026-10-10-specky-1BY4F4Z.md
- **Kind**: feature
- **Priority**: normal
- **Affects**: general (nowe wymaganie Specky VA-REC-5; crates/audio/src/silence.rs, kontroler)
- **Summary**: Pauzy wewnątrz nagrania dłuższe niż 1,5 s są skracane do 0,5 s przed transkrypcją, żeby Whisper nie halucynował w oknach bez mowy; brzegi jak dotąd.
- **Modified pending tasks**: none
- **New rework phase**: none
- **New tasks**: 7.1, 7.2 (Faza 7)

### 2026-10-10 2026-10-10-specky-9VFYND0.md
- **Kind**: feature
- **Priority**: normal
- **Affects**: general (nowe wymaganie Specky VA-REC-6; recorder, va-config, kontroler, messages.rs)
- **Summary**: Limit nagrania domyślnie 600 s; po jego osiągnięciu nagrywarka zgłasza limit, kontroler kończy nagranie jak po ctrl+cmd+s (transkrypcja do schowka), a użytkownik dostaje powiadomienie — dziś nadwyżka ginęła po cichu.
- **Modified pending tasks**: none
- **New rework phase**: none
- **New tasks**: 7.3, 7.4 (Faza 7)

## Visual artifacts

<!--
Section filled in ONLY when the `## Visual artifacts` section in telesma/config.md has
`Screenshots: yes` or `Recordings: yes`. Full procedure: section 0.5 of the instructions.

Row format:
| Phase | Type | ID | Artifact file | Spec source |
|-------|------|----|--------------:|-------------|
| 1     | screen | B1_ProjectOverview | telesma/artifacts/screens/B1_ProjectOverview.png | telesma/spec/ux_ui/screens-b1-b2.jsx:52-200 |
| 1     | flow   | step-1             | telesma/artifacts/flows/step-1-sign-up.webm      | telesma/spec/APP_FLOW.md:11-30 |

Info warnings (no APP_FLOW.md despite Recordings: yes, no flow_files, etc.) go
below the table with the prefix `> ⚠ Info:` so the user can see what happened "silently".
-->

| Phase | Type | ID | Artifact file | Spec source |
|------|-----|----|----------------|-------------|
| | | | | |

## Problem log

<!-- Format: [date] Task X.X: problem description - fix/status -->
<!-- We do NOT keep a list of changed files — git knows it better: git log --stat -->
- [2026-11-04] Task 10.7: test całego toru spotkania (`crates/core/tests/meeting_real.rs`, prawdziwy mikrofon + tap + Whisper) dał pusty wynik. Przyczyna (błąd z 10.5, wykryty przez Telesmę przed testem ręcznym): blok IO urządzenia zagregowanego dostaje próbki w częstotliwości zegara wyjścia systemowego (słuchawki PXC 550 w trybie Bluetooth 16 kHz), a kod przyjmował 48 kHz z formatu tapu — 3,6 s dźwięku zamieniało się w 1,46 s szumu. Poprawka: częstotliwość z `kAudioDevicePropertyNominalSampleRate` prywatnego urządzenia; test tapu sprawdza teraz długość przechwycenia (3,4–6 s dla fixture 3,6 s), nie tylko „> 1 s”. Drugi problem: wątek transkrypcji z modelem nie był dołączany przy zamykaniu kontrolera, więc proces testu kończył się asercją GGML (kod 101); `TranscriptionWorker` dołącza wątek w `Drop`.
- [2026-10-13] Task 5.Z1.1: zmiana testu `after_recording_the_dot_is_gray_again` (indicator.rs) → `transcribing_shows_yellow_dot` + `after_transcription_or_failure_the_dot_is_gray_again`. Powód: VA-UI-1 v2 kryterium 01M4EKHD1PJ32E01218FSE9G8F (telesma/changes/processed/2026-10-13-specky-5FJTRC.md) — w trakcie transkrypcji kółko żółte, nie szare. Kod kontrolera bez zmian: kolejność (Idle dopiero po zapisie do schowka) i ignorowanie Startu w Transcribing już działały, doszły testy ze znacznikami; mutacje (szare w Transcribing, Start w Transcribing, Idle przed zapisem) czerwone.
- [2026-10-13] Blokada z poprzedniej sesji rozwiązana: właściciel dopisał cargo (test, clippy, fmt, check, build) i `gh pr merge` do allowlisty; telesma/state/BLOCKED.md usunięty.
- [2026-10-13] Start sesji: każde wywołanie `cargo` (także `cargo --version`, z `export PATH` i pełną ścieżką `~/.cargo/bin/cargo`) zwraca „requires approval” od sędziego uprawnień; w trybie bez nadzoru nie ma kto zatwierdzić. Regresja Faz 7–9 i 5.Z1.1 wstrzymane — TELESMA BLOCKED (telesma/state/BLOCKED.md).
- [2026-10-13] Zadania 7.7/9.10: powiadomienia (gotowa transkrypcja, limit) nie pojawiały się, także w Centrum powiadomień — przyczyna: powiadomienia zablokowane w ustawieniach macOS u właściciela; po odblokowaniu widoczne. Zgłoszenie błędu (telesma/changes/2026-10-13-powiadomienia-niewidoczne.md) wycofane przed przetworzeniem. Powiadomienia są podpisane jako VoiceAsystent (osobna pozycja w Ustawieniach → Powiadomienia, potwierdzone zrzutem właściciela).
- [2026-10-10] Task 7.4: ostrzeżenie kontroli `testy/kod-bez-testu` (app.rs, main.rs, messages.rs) — testy są w `#[cfg(test)] mod tests` tych samych plików (znana pułapka), nie do naprawy przenoszeniem. Powiadomienie systemowe przy limicie niezweryfikowane wizualnie — test ręczny 7.7 (limit 20 s w config.toml).
- [2026-10-10] Task 7.3: zmiana testów `partial_file_keeps_defaults_for_missing_fields` (va-config) i `config_without_file_prints_defaults` (va-dev) — oczekiwany domyślny limit 300 → 600 s. Powód: kryterium 01M4K06AY61DWRJ5R2DFNDJD6W (VA-REC-6, telesma/changes/processed/2026-10-10-specky-9VFYND0.md) ustala domyślne 600 s. Spóźniony sygnał limitu z poprzedniego nagrania jest odrzucany po numerze nagrania (test `stale_limit_signal_from_previous_recording_is_ignored`).
- [2026-10-10] Task 7.2: zmiana testu `transcribe_keeps_both_sentences_around_a_long_pause_without_extra_phrases` — słowo kluczowe «biurze» → «jutro»: syntezator `say` (Zosia) wymawia „biurze” tak, że Whisper daje „Białże”; kryterium 01M4K06AR8TJXY24K05X0MNBJC (VA-REC-5, telesma/changes/processed/2026-10-10-specky-1BY4F4Z.md) wymaga obu zdań bez dodatkowych fraz, nie rozpoznania konkretnego słowa. Wynik prawdziwej transkrypcji: „Dzisiaj jest piękna pogoda, idę na spacer do parku. Jutro rano mam spotkanie w Białże o 9.00.” — 17 słów, bez halucynacji w 40 s pauzy.
- [2026-10-10] Start Fazy 7: pełny suite po przetworzeniu telesma/changes/ nie powtórzony — kod (crates/apps/scripts/Cargo) bajt w bajt ten sam, co w regresji Fazy 6 z 2026-10-10 (126/0); zmiany dotyczyły tylko telesma/, docs/ i telesma/changes/.
- [2026-10-10] Task 9.11: pełny suite po 9.9 dał 209/1 — `model_loads_once_on_metal_and_transcribes_polish_and_english` (VA-STT-1) padał 4/4 przy całym pliku `whisper_model.rs`, sam przechodził. Przyczyna: subskrybent `tracing` przypięty do wątku testu, a logi whisper.cpp idą przez globalny hook; równoległe testy modelu dodane w 9.4 i 9.7 gubiły linie o Metalu w buforze (asercje w liniach 85 i 93). Kod aplikacji bez zmian; test przeniesiony do osobnego pliku (osobny proces), asercje bez zmian. Regresja izolacji testów — powstała w 9.4, niewidoczna, bo przebiegi zadań uruchamiały tylko testy zadania. Ostrzeżenie `testy/mniej-asercji` (whisper_model.rs 18 → 10) — asercje nie usunięte, tylko przeniesione do whisper_metal.rs razem z testem; `skip-z-powodem` jak wcześniej (model i GPU).
- [2026-10-10] [check] PR #58 — CI `test`: `release_check_accepts_small_image_without_model` (z 9.9) padł z kodem 1 bez komunikatu — `hdiutil attach -quiet` przy `set -e` kończył `check-dmg.sh` po cichu; pierwszy push tej samej gałęzi był zielony, lokalnie 1 porażka na 3 przebiegi dmg.rs. Po usunięciu `-quiet` przyczyna potwierdzona lokalnie: `hdiutil: attach failed - Zasoby czasowo niedostępne` przy kilku testach obrazów naraz (same ponawianie 3× nie wystarczyło, 1 na 5 przebiegów czerwony). Poprawka: testy obrazów szeregowane (mutex w dmg.rs dla `cargo test`, grupa nextest `obrazy-dysku` z max-threads 1 w `.config/nextest.toml` dla CI), plus 3 próby z błędem `hdiutil` na stderr w skryptach (przydatne też w workflow wydania); nowy test obrazu nie do zamontowania. Po poprawce szybki zestaw 6× zielony.
