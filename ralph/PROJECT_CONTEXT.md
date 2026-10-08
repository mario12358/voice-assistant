# PROJECT_CONTEXT.md

Ten plik przechowuje kontekst projektu który przetrwa `/compact`.

## Stack technologiczny

- Język: Rust (workspace Cargo)
- Platforma: wyłącznie macOS (Apple Silicon), dystrybucja .dmg
- STT: whisper-rs + Whisper large-v3-turbo (GGML), backend Metal
- UI: aplikacja paska menu (tray-icon + muda + tao), skróty global-hotkey, schowek arboard
- Testy: cargo test (testy z modelem/GPU/mikrofonem `#[ignore]`)

## Architektura

Workspace Cargo: biblioteki w `crates/`, binarki w `apps/`. Kod (identyfikatory) po angielsku, komunikaty dla użytkownika po polsku.

| Crate | Ścieżka | Odpowiedzialność |
|-------|---------|------------------|
| `va-config` | crates/config | `Config` (serde/toml): mikrofon, język, progi ciszy, limit nagrania, ścieżka modelu; odczyt/zapis; ścieżki macOS (`Paths`) |
| `va-audio` | crates/audio | trait `AudioHost` (lista urządzeń, domyślne), wybór mikrofonu z fallbackiem, nagrywanie (cpal) → 16 kHz mono f32 (rubato), przycinanie ciszy |
| `va-model` | crates/model | pobieranie GGML large-v3-turbo (HTTP Range, `.part` → rename), SHA-256, stan „brak/pobieranie/gotowy” |
| `va-stt` | crates/stt | trait `SpeechToText`, implementacja whisper-rs (feature `metal`), sprawdzenie GPU (`GpuProbe`), mock testowy |
| `va-clipboard` | crates/clipboard | trait `TextSink`, implementacja arboard; pusty tekst nie zmienia schowka |
| `va-core` | crates/core | automat stanów (Idle/Recording/Transcribing/Error), `Controller` łączący audio → STT → schowek, zdarzenia stanu dla UI; bez GUI |
| `voice-asystent` | apps/voice-asystent | aplikacja paska menu: tao (wątek główny), tray-icon + muda, global-hotkey, okablowanie zależności |
| `va-dev` | apps/va-dev | CLI deweloperskie (clap): `mic-test`, `transcribe`, `model-download` |

Konwencje:
- **Błędy**: każdy crate biblioteczny ma własny `enum Error` (thiserror) i `type Result<T>`; binarki używają `anyhow` z `.context(...)`. Komunikat dla użytkownika powstaje w warstwie UI (mapowanie błędu → tekst PL), nie w bibliotekach.
- **Wymienne elementy za traitami** (`AudioHost`, `Recorder`, `SpeechToText`, `TextSink`, `GpuProbe`, `ModelSource`), wstrzykiwane do `Controller` jako `Box<dyn Trait + Send>`; testy używają ręcznych fałszywek z crate'u `va-core` (`#[cfg(test)]`) albo modułów `testing` za feature `testing`.
- **Logowanie**: `tracing`; spany `recording`, `transcription` z czasem; treść transkrypcji tylko `debug!`; audio nigdy na dysk (poza jawnym `va-dev mic-test --save`).
- **Ścieżki macOS**: konfiguracja `~/Library/Application Support/VoiceAsystent/config.toml`, model `…/VoiceAsystent/models/ggml-large-v3-turbo.bin`, logi `~/Library/Logs/VoiceAsystent/`.
- **Platforma**: `va-core` i obie binarki mają `#[cfg(not(target_os = "macos"))] compile_error!(...)`; brak konfiguracji CI/budowania dla innych systemów.
- **Testy**: jednostkowe w module `#[cfg(test)] mod tests`, integracyjne w `crates/<x>/tests/`; fixtures WAV w `tests/fixtures/` (katalog workspace). Nazwy testów opisują zachowanie (`empty_transcript_leaves_clipboard_untouched`). Testy wymagające modelu, GPU, mikrofonu lub sesji graficznej: `#[ignore = "wymaga …"]`, uruchamiane lokalnie `cargo test --workspace -- --include-ignored`. Kryteria Specky: `// specky: crit <id>` w linii nad `#[test]`.
- **Budowanie**: GPU tylko przez feature `metal` w `va-stt` (włączony domyślnie w binarkach); budowanie bez Metal nie jest wspierane w wydaniu.

## Kluczowe decyzje

<!-- Tabela rośnie z każdą sesją, a plik trafia w całości do promptu startowego, więc         -->
<!-- ralph-start.sh trzyma w niej 40 ostatnich wierszy — starsze idą do ralph/CONTEXT_ARCHIVE.md -->
<!-- (write-only). Decyzję wciąż OBOWIĄZUJĄCĄ — taką, której złamanie zepsułoby projekt —     -->
<!-- oznacz 📌 w kolumnie Decyzja: wiersz z 📌 nie podlega rotacji nigdy.                      -->

| Data | Decyzja | Powód |
|------|---------|-------|
| 2026-10-08 | 📌 Aplikacja wyłącznie na macOS, instalowana z .dmg; Windows/Linux poza zakresem | decyzja właściciela |
| 2026-10-08 | 📌 Specky (projekt 01M4EJNFTHDZ3ECMHT7425APR0) jest jedynym źródłem wymagań; `wymagania.md` to tylko wsad importu | decyzja właściciela |
| 2026-10-08 | 📌 Model rozpoznawania mowy tylko na GPU (Metal), bez fallbacku CPU | spec/WYTYCZNE_TECHNICZNE.md |
| 2026-10-08 | 📌 Model large-v3-turbo NIE jest w instalatorze; aplikacja pobiera go po instalacji do ~/Library/Application Support/VoiceAsystent/models/ (VA-MODEL-1) | decyzja właściciela: mały .dmg |
| 2026-10-08 | 📌 Wątek główny = pętla tao (tray-icon, global-hotkey); logika w `va-core` bez zależności od GUI, testowalna na wstrzykniętych zdarzeniach | wymóg AppKit + testowalność |
| 2026-10-08 | Skróty przez global-hotkey (Carbon RegisterEventHotKey) — nie wymaga uprawnienia Dostępność | prostsza instalacja |
| 2026-10-08 | Lewe kliknięcie ikony = start/stop, menu (mikrofon, zakończ) pod prawym kliknięciem | VA-UI-2 + VA-REC-4 bez konfliktu |
| 2026-10-08 | Logi: JSON do dziennego pliku `~/Library/Logs/VoiceAsystent/voice-asystent.log.*` + tekst na stderr, spany z czasem (FmtSpan::CLOSE); bez OTLP/eksportu — aplikacja lokalna, po pobraniu modelu zero ruchu sieciowego | odstępstwo od observability.md uzasadnione VA-MODEL-1 |
| 2026-10-08 | 📌 Treść transkrypcji logujemy tylko przez `va_core::logging::transcription_finished` (debug); na info tylko długość i czas | prywatność |
| 2026-10-08 | Język transkrypcji domyślnie `auto` (konfigurowalny pl/en) | wymagania nie określają języka |
| 2026-10-08 | `va_audio::SilenceParams` niezależne od `va-config` (audio nie zależy od konfiguracji); mapowanie `SilenceConfig` → `SilenceParams` w miejscu wywołania (va-dev, kontroler) | crate audio bez zależności od konfiguracji |
| 2026-10-08 | Fixtures WAV w `tests/fixtures/` generowane skryptem `generate_silence_fixtures.py` (`say` macOS, 16 kHz mono) — granice mowy znane co do próbki | powtarzalność |
| 2026-10-08 | Wykrywanie Metal przez objc2-metal (`MTLCreateSystemDefaultDevice`, bezpieczne API, bez `unsafe`); brak urządzenia lub build bez feature `metal` → `Error::GpuUnavailable`/`MetalNotBuilt`, transkrypcja zablokowana, aplikacja działa dalej | VA-STT-2 |
| 2026-10-08 | 📌 Specky nie czyta znaczników w Ruście (tylko .py/.js/.ts): pracujemy bez dowodów kryteriów z CI; znaczniki `// specky: crit <id>` w testach ZOSTAJĄ (nad `fn`, pod `#[test]`), kryteria właściciel odhacza ręcznie w komentarzu Specky na PR; „brak testu” w tym komentarzu nie jest błędem do naprawy | decyzja właściciela |
| 2026-10-08 | Model z Hugging Face `ggerganov/whisper.cpp` `ggml-large-v3-turbo.bin` (1 624 555 275 B, SHA-256 1fc70f77…bc69 z API LFS); HTTP przez ureq 3 bez gzip (Range + kompresja się gryzą), `check()` liczy SHA-256 przy każdym starcie | VA-MODEL-1 |
| 2026-10-09 | whisper-rs 0.16 z `tracing_backend` (logi whisper.cpp/ggml w tracing); `WhisperStt::load` wymaga `GpuReady` i ustawia `use_gpu(true)`; dowód Metal w teście = linie whisper.cpp `whisper_backend_init_gpu: device 0: Metal` i `Metal total size`, nie własny log | VA-STT-2 K1 |
| 2026-10-08 | Zakres: nagranie → transkrypcja → schowek; bez LLM, TTS, wpisywania do okna (wcześniejszy plan Linux/CUDA porzucony) | wymagania.md |

## Znane problemy i rozwiązania

<!-- Znane pułapki: destylowane z Logu problemów przy checkpoincie końca fazy (sekcja 7    -->
<!-- instrukcji). Tylko wzorce POWTARZALNE — błąd jednorazowy zostaje w Logu problemów     -->
<!-- w REPORT.md. Jeden wiersz = jedna pułapka, w JEDNEJ linii i zwięźle: plik idzie       -->
<!-- w całości do promptu każdej sesji.                                                     -->
<!-- ralph-start.sh trzyma tu 30 ostatnich wierszy, starsze → ralph/CONTEXT_ARCHIVE.md.    -->
<!-- Pułapkę, która WRACA mimo zapisu (powtórzyła się ≥2 razy), oznacz 📌 — taki wiersz    -->
<!-- nie rotuje się nigdy. Jednorazową wpadkę zostaw bez znacznika.                        -->

| Problem | Rozwiązanie |
|---------|-------------|
| Specky contract check czerwony na PR zadania bez wymagania (narzędzia, stan fazy) | w commicie i w `--body` squasha linia `Specky-Req: none` |
| `cargo` poza PATH w nowym shellu agenta | w Bash `export PATH="$HOME/.cargo/bin:$PATH"`; hook kontroli dokłada ~/.cargo/bin sam (naprawione 2026-10-09) |
| Status kontroli PR czerwony dla commita SPRZED ostatniego (amend przelicza tylko HEAD) | `git reset --soft <baza gałęzi>` + ponowny commit przez hook, potem `push --force-with-lease` |
| Pierwszy commit gałęzi bez dowodu kontroli (status PR czerwony) | `git commit --amend --no-edit` + `git push --force-with-lease` (19.4) |
| Kontrola hooka odrzuca komendę, w której przed `git commit` stoi zapis plików | zapis plików i `git add && git commit` zawsze osobnymi wywołaniami |

## Zależności między komponentami

- Wątek główny (wymóg macOS): pętla tao, ikona i menu (tray-icon/muda), skróty (global-hotkey). Zdarzenia kliknięcia/skrótu → `Command::{Start, Stop}` → kanał do `Controller`.
- Wątek `Controller` (va-core): trzyma automat stanów; Start → `Recorder` (cpal `Stream` jest `!Send`, więc nagrywanie żyje we własnym wątku audio); Stop → bufor → przycięcie ciszy → `SpeechToText` → `TextSink`.
- `Controller` publikuje `StateChanged(State)` przez `EventLoopProxy` → UI zmienia ikonę (szare/czerwone kółko) i podpowiedź.
- Start: `va-config` (odczyt) → `GpuProbe` → `va-model` (jest model? jeśli nie, pobieranie w tle z postępem w menu) → załadowanie `SpeechToText` raz → stan Idle.
- Wybór mikrofonu w menu → `va-config` (zapis) → `Controller` używa go od następnego nagrania.

## Aktualny stan

<!-- STAN BIEŻĄCY — NADPISUJ. To ma być odpowiedź na „gdzie jesteśmy DZIŚ", a nie kronika  -->
<!-- tur: historia realizacji ma własne miejsce w REPORT.md i tam się rotuje, a tutaj nie  -->
<!-- rotuje się nic. Dopisywanie kolejnych akapitów „tura z 19.08 domknięta" zamienia tę   -->
<!-- sekcję w drugi, nieograniczony raport w prompcie każdej sesji.                        -->

- Ostatnie ukończone: 3.3 (whisper-rs na Metal); 3.4 (`va-dev transcribe`) w PR #10
- Następne: po merge #10 regresja Fazy 3 na main + tag; potem 4.1 (schowek)
- Blokery: brak
