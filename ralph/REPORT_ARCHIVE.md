# ralph/REPORT_ARCHIVE.md — archiwum REPORT.md

Starsze wpisy przenoszone automatycznie przez ralph-start.sh przy starcie.
Write-only — plik nie jest ładowany do żadnego promptu.

## Rotacja 2026-10-09

### Stan testów

**Regresja po Fazie 2 (2026-10-08) — ZIELONA:** na main (1d996a2) `cargo test --workspace -- --include-ignored` 49/0 (z 3 testami mikrofonu), clippy `-D warnings` czysto, `cargo fmt --check` czysto. Względem 17/0 po Fazie 1: +32 testy. Tag `ralph/faza-2`.

**Regresja po Fazie 1 (2026-10-08) — ZIELONA:** `cargo test --workspace -- --include-ignored` 17/0 (0 ignored), clippy `-D warnings` czysto, `cargo fmt --check` czysto. Tag `ralph/faza-1`.

### Historia realizacji

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

### Log problemów

- [2026-10-08] Start sesji: PLAN (Linux/CUDA, LLM, TTS) przeczy wymagania.md (macOS, schowek, pasek menu). Specky pusty, brak repo git i toolchainu Rust. RALPH BLOCKED, czekam na decyzję właściciela.

- [2026-10-08] Odpowiedź właściciela: tylko macOS + .dmg, Specky jedynym źródłem wymagań, git z origin gotowy. Plan przepisany, 11 wymagań zaproponowanych w Specky (wsad 01M4EK43A6GBPR63H0FV0VBCV0). Zostało: instalacja Rust, akceptacja wymagań.

- [2026-10-08] Zadanie 2.3: kryterium VA-REC-3 «pusta transkrypcja nie zmienia schowka» (crit 01M4EKHCXW1S92G9Y6AS115MW9) nie ma jeszcze testu z `specky: crit` — 2.3 daje tylko pusty wynik dla ciszy; test z schowkiem i brakiem wywołania STT powstaje w 4.1/4.3.

- [2026-10-08] Zadanie 2.3: pierwszy commit gałęzi bez dowodu kontroli (status PR czerwony) — naprawione `git commit --amend --no-edit` + push (19.4).

- [2026-10-08] Zadanie 2.4: kontrola lint-typy ostrzega „linter niedostępny (cargo)” — hook nie widzi `~/.cargo/bin` w PATH, więc commit przeszedł BEZ clippy/fmt w hooku. Clippy `-D warnings` i fmt uruchomione ręcznie — czysto. Do decyzji właściciela: PATH dla hooka albo pełna ścieżka w config.

- [2026-10-08] [check] PR #2 — Specky contract check: brak trailera Specky-Req (zadanie narzędziowe bez wymagania) — dopisany `Specky-Req: none` w commicie i w treści squasha.

- [2026-10-08] [check] PR #5 — Specky contract check: kryterium VA-STT-2 K2 «brak testu» mimo `// specky: crit` nad `#[test]` (Specky nie przeskakuje atrybutów Rusta?) — próba 1: znacznik między `#[test]` a `fn` — bez zmian (has_test=false). Podejrzenie: JUnit z nextest ma classname = nazwa crate'u, bez ścieżki pliku, więc Specky nie łączy testu z plikiem źródłowym. RALPH BLOCKED.

- [2026-10-08] Blokada rozwiązana: Specky czyta znaczniki tylko w .py/.js/.ts — właściciel wybrał pracę bez dowodów z CI, znaczniki zostają, kryteria odhaczane ręcznie na PR. ralph/BLOCKED.md usunięty.

- [2026-10-08] Zadanie 3.1: AC «wejście — va-dev transcribe» domknięte w 3.4 (`transcribe::run` woła `require_metal`; test transcribe_without_model_points_to_model_download).

- [2026-10-08] Tag `ralph/faza-2`: kontrola `zaleznosci` nie wykonała się (UnicodeDecodeError 0xfa — prawdopodobnie czyta binarne fixtures WAV jako tekst); tag przeszedł bez niej. Do zgłoszenia właścicielowi.

- [2026-10-08] Blokada rozwiązana: Rust 1.99 zainstalowany, wsad 11 wymagań zaakceptowany, kotwice w planie. Model pobierany po instalacji: dodano 5.6 i propozycję VA-MODEL-1 w Specky.

## Rotacja 2026-10-11

### Stan testów

**Przebieg po zadaniach 9.1–9.11 (2026-10-10) — ZIELONY (main 4e23e64, nie regresja fazy — czekają ⛔ 7.7, ⛔ 8.5 i ⛔ 9.10):** `cargo test --workspace -- --include-ignored` 211/0 dwa razy pod rząd (w tym prawdziwy model pełny i q5_0, szum, mieszane PL, Metal, mikrofon, build release, obrazy .dmg), clippy `-D warnings` czysto, `cargo fmt --check` czysto; CI nextest 197/197. Względem 171/0 po 8.4: +40 testów. Pierwszy przebieg po 9.9 dał 209/1 — niestabilny test logów Metal i montowanie .dmg naprawione w 9.11. Bez tagów `ralph/faza-7..9` do wyników testów ręcznych.

**Przebieg po zadaniach 6.1–6.3 (2026-10-09) — ZIELONY (main 5f13dab, nie regresja fazy — czekało ⛔ 6.4):** `cargo test --workspace -- --include-ignored` 126/0 (w tym build release i bundle, montowanie .dmg, mikrofon, GPU Metal, model, schowek), clippy `-D warnings` czysto, `cargo fmt --check` czysto. Względem 119/0 po Fazie 5: +7 testów (bundle.rs 5, dmg.rs 2). Bez tagu `ralph/faza-6` do wyniku testu ręcznego 6.4.

**Regresja po Fazie 5 (2026-10-09) — ZIELONA:** na main (f76c960) `cargo test --workspace -- --include-ignored` 119/0 (z mikrofonem, GPU Metal, modelem i prawdziwym schowkiem), clippy `-D warnings` czysto, `cargo fmt --check` czysto. Względem 88/0 po Fazie 4: +31 testów. Tag `ralph/faza-5`. Zachowanie GUI (kliknięcia, skróty w innych aplikacjach, powiadomienia, wygląd ikony) poza zasięgiem testów automatycznych — scenariusze ręczne.

**Regresja po Fazie 4 (2026-10-09) — ZIELONA:** na main (96956d8) `cargo test --workspace -- --include-ignored` 88/0 (z mikrofonem, GPU Metal, modelem i prawdziwym schowkiem), clippy `-D warnings` czysto, `cargo fmt --check` czysto. Względem 70/0 po Fazie 3: +18 testów. Tag `ralph/faza-4`.

**Regresja po Fazie 3 (2026-10-09) — ZIELONA:** na main (ec17148) `cargo test --workspace -- --include-ignored` 70/0 (z testami mikrofonu, GPU Metal i prawdziwego modelu), clippy `-D warnings` czysto, `cargo fmt --check` czysto. Względem 49/0 po Fazie 2: +21 testów. Tag `ralph/faza-3`.

### Historia zmian

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

### Historia realizacji

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

| 9.11 Naprawa niestabilnego testu logów Metal: `model_loads_once_on_metal…` w osobnym pliku `whisper_metal.rs` (osobny proces), pomocniki w `tests/common/mod.rs`; ponawianie `hdiutil attach`/`create` z widocznym błędem w `check-dmg.sh`, `build-dmg.sh` i teście montowania + test 9.11 | ✅ PR #58 | va-stt z `--include-ignored` 3× zielony (12/0); dmg.rs 19× i szybki zestaw 6× zielone po szeregowaniu testów obrazów; mutacje `use_gpu(false)` i brak `exit 1` po nieudanym montowaniu czerwone | 3 | 4e23e64 | 2026-10-10 |

| 9.9 `.github/workflows/release.yml` (tag v* → testy → build-dmg → check-dmg → `gh release create --notes-from-tag`), `scripts/check-dmg.sh` (rozmiar < 20 MB, brak `*.bin`/`ggml*`), RUNBOOK sekcja wydania + test 9.9 | ✅ PR #57 | 196/0 + 14 ignored; prawdziwy obraz release 4,05 MB przechodzi kontrolę; YAML sprawdzony Ruby (actionlint niedostępny); mutacja (brak szukania modelu) czerwona | 1 | 586080c | 2026-10-10 |

### Log problemów

- [2026-10-10] Zadanie 9.9: workflow `release.yml` nie da się uruchomić bez wypchnięcia tagu — weryfikacja przy następnym wydaniu, które właściciel wypchnie (kryteria 1–2 VA-CI-1 do testu ręcznego 9.10). Lokalnie sprawdzone: składnia YAML (Ruby; actionlint niedostępny), ten sam łańcuch build-dmg → check-dmg na obrazie release (4,05 MB, bez modelu) i testy check-dmg (poprawny → 0, z plikiem modelu → 1, ponad limit → 1). Tagi v0.4.0–v0.6.0 nadal tylko lokalnie.

- [2026-10-10] Zadanie 9.8: zmiana testów `status_line_shows_name_size_and_state_for_every_download_state`, `size_is_rounded_…` i `removal_needs_a_confirmation_…` — rozmiar 1 624 555 275 B pokazywany jako „1,6 GB” zamiast „1,7 GB” (zaokrąglanie do najbliższej dziesiątej zamiast w górę). Powód: kryterium 01M4KD15M5E90XXM60PPS2S33J (VA-MODEL-4) wymaga etykiety „Pełny (1,6 GB)”, a Finder pokazuje 1,62 GB; scenariusze v0.6.0 mówią 1,7 GB — w scenariuszach v0.7.0 poprawne. Przełączenie na pobrany już wariant pokazuje przez czas ładowania (1–2 s) „Pobieranie modelu… 0%” — ta sama ścieżka co pobieranie (download zwraca od razu, gdy plik zweryfikowany); świadomie, do testu ręcznego 9.10. `SettingChange::command` zwraca `Option` (wariant nie ma polecenia dla kontrolera).

- [2026-10-10] Zadanie 9.4: `no_speech_probability()` whisper.cpp dla large-v3-turbo wynosi 0,000 dla każdego segmentu, także szumu (model uważa szum za mowę), więc sam próg no-speech nic nie odrzuca. Pomiar średniej pewności tokenów tekstu: szum 6 s → „so” 0,048; mowa w fixtures 0,90–0,997 (pojedyncze tokeny do 0,24). Dodany próg `min_segment_confidence = 0,3` na średnią segmentu. Uwaga: `cargo test -p va-stt` bez `--features metal` nie ładuje modelu (cecha włączana tylko przez binarki; w `--workspace` cechy się sumują) — testy modelu uruchamiać z `--workspace` albo `--features metal`.

- [2026-10-10] [check] PR #51 — test: wyścig w atrapie `FakeRecorder::reach_limit_of` (zadanie 7.3) — kontroler publikuje `StateChanged(Recording)` przed `Recorder::start`, więc na runnerze CI test wołał sygnał limitu, zanim atrapa go zarejestrowała (indeks poza zakresem, zatruta blokada). Lokalnie zielony 3×, w CI flaky. Naprawa: pomocnik czeka do `WAIT` na rejestrację nagrania; asercje testu bez zmian, kod produkcyjny bez zmian. Wykryte przez: test (CI).

- [2026-10-10] Zadanie 9.3: zmiana testów kontrolera `stop_puts_transcript_in_clipboard_and_reports_states_to_ui` (w sekwencji doszło `TranscriptReady` po `Delivered`) i `limit_reached_…` (filtr pomija też `TranscriptReady`) oraz testu `full_file_overrides_every_field` (nowe pola konfiguracji) — VA-UX-1, kryteria 01M4KD152NZM9VP8S1V8VX1E6R i 01M4KD152NK6764Z8AMZPSHQQN; asercje stanów i schowka bez zmian. Przy okazji: log zdarzeń w pętli tao wypisywał dotąd całe zdarzenie (`?event`) na poziomie debug, czyli także treść historii — teraz tylko rodzaj (`event_kind`), co domyka też kryterium 6 VA-HIST-1 przy debug. `app::run` dostał `AppSettings` (clippy: za dużo argumentów).

- [2026-10-10] Zadanie 9.2: znacznik jako prosty tekst `rozmiar mtime_ns sha256` (bez nowej zależności serde w va-model) w pliku `ggml-large-v3-turbo.bin.verified`; plan zakładał `.verified.json` — format nie ma znaczenia dla kryteriów. Test pominięcia sumy działa przez podmianę treści przy tym samym rozmiarze i dacie (check nadal Ready) — to jednocześnie dokumentuje świadomy kompromis VA-PERF-1. `va-dev model-download` korzysta z mechanizmu przez `ModelStore::download` → `check`. Kryterium 3 (ikona ≤ 1,5 s) zmierzone pośrednio: `--self-check` release 0,04 s przy aktualnym znaczniku; ikona w teście ręcznym 9.10.

- [2026-10-10] Zadanie 8.4: ostrzeżenie kontroli `testy/kod-bez-testu` (5 plików app) — testy w `#[cfg(test)] mod tests` (znana pułapka). Pełna ścieżka Usuń → Pobierz ponownie → nagranie bez restartu wymaga ponownego pobrania 1,6 GB i kliknięć w prawdziwym menu — test ręczny 8.5; automat potwierdzenia i automat pobierania (Missing → Retry → Downloading → Ready) pokryte testami jednostkowymi, ścieżka ModelLoaded → nowy kontroler to istniejąca ścieżka z 5.6.

- [2026-10-10] Zadanie 8.3: ostrzeżenie kontroli `testy/kod-bez-testu` (6 plików app) — testy w `#[cfg(test)] mod tests` (znana pułapka). Rozmiar modelu w menu zaokrąglany w górę do dziesiątych GB (1 624 555 275 B → „1,7 GB”); „Pokaż w Finderze” i wygląd podmenu — test ręczny 8.5.

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

## Rotacja 2026-10-13

### Historia realizacji

| Zadanie | Status | Testy | Próby | Commit | Czas |
|---------|--------|-------|-------|--------|------|
| 9.8 Wybór wariantu w podmenu „Model” (pozycje Pełny 1,6 GB / q5_0 0,6 GB z zaznaczeniem, przełączenie: zapis `model_variant` → `DownloadEvent::Switch` → pobranie albo samo załadowanie → `ModelLoaded`, „Usuń nieużywany wariant”), rozmiary zaokrąglane do najbliższej dziesiątej; README/RUNBOOK fazy 9 + test 9.8 | ✅ PR #56 | 194/0 + 14 ignored; mutacja (przełączenie bez zapisu do konfiguracji) czerwona; aplikacja startuje lokalnie | 1 | e97da01 | 2026-10-10 |

| 9.7 Wariant q5_0: `ModelVariant` (`model_variant` w config.toml, domyślnie full), `LARGE_V3_TURBO_Q5_0` (574 041 195 B, SHA-256 z LFS HF) i `spec_for`, `ModelSource::Store(store, wariant)`, pobieranie i usuwanie per wariant, `va-dev model-download --variant` + test 9.7 | ✅ PR #55 | 189/0 + 14 ignored; q5_0 pobrany (54 s) i rozpoznaje speech_pl; mutacja (wariant ignorowany) czerwona | 1 | af22fbb | 2026-10-10 |

| 9.6 „Uruchamiaj przy logowaniu”: `login_item.rs` (plist LaunchAgent `io.github.mario12358.voiceasystent`, RunAtLoad, ścieżka bundla z `current_exe`, escape XML), pozycja z zaznaczeniem w podmenu Ustawienia, nieaktywna poza bundlem; `Paths::launch_agents_dir` + test 9.6 | ✅ PR #54 | 186/0 + 13 ignored; plist przechodzi `plutil -lint`; mutacja (zaznaczenie bez sprawdzania ścieżki) czerwona | 1 | 0b4a713 | 2026-10-10 |

### Log problemów

- [2026-10-10] Zadanie 8.2: `DownloadEvent::Removed` i `DownloadState::can_remove` mają tymczasowo `#[allow(dead_code)]` — konstruowane/używane dopiero przez podmenu „Model” (8.3) i usuwanie (8.4); atrybuty do zdjęcia w tych zadaniach (clippy `-D warnings` nie przepuszcza nieużywanych symboli, a dzielenie automatu między PR-y bez tego byłoby niemożliwe).
