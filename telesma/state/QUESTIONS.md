# telesma/state/QUESTIONS.md - Project clarification

## Status

- **State**: ukończony
- **Total questions**: 18
- **Answered**: 18
- **Assumptions**: 12
- **Contradictions**: 0
- **Generated on**: 2026-10-08

## Analyzed documents

- `telesma/spec/WYTYCZNE_TECHNICZNE.md` (86 linii) — jedyne źródło merytoryczne: Rust, model large-v3-turbo, tylko GPU
- `telesma/spec/APP_FLOW.md` (101 linii) — niewypełniony szablon (marker `RALPH_APP_FLOW_TEMPLATE`)
- `telesma/spec/ux_ui/LINKS.md` (28 linii) — niewypełniony szablon (marker `RALPH_UX_LINKS_TEMPLATE`)

Ocena jakości: kompletność ~5%, jednoznaczność niska → duża liczba pytań (szczegóły w `telesma/state/SPEC_INDEX.md`).

## Contradictions in the documentation

<!-- Brak wykrytych sprzeczności -->

## Assumptions

### A1: Model „large-v3-turbo" to OpenAI Whisper large-v3-turbo (rozpoznawanie mowy, STT)
- **Category**: technologia
- **Source**: WYTYCZNE_TECHNICZNE.md:30
- **Rationale**: To jedyny powszechnie znany model o tej nazwie; w połączeniu z nazwą projektu „voice_asystent" oznacza warstwę speech-to-text.
- **Status**: ✅

### A2: Szablonowy `telesma/spec/APP_FLOW.md` (rejestracja, projekty web) NIE jest źródłem wymagań
- **Category**: wymagania
- **Source**: APP_FLOW.md:1 (marker szablonu), APP_FLOW.md:36-81
- **Rationale**: Plik ma marker szablonu, a przykładowe flow (konta, projekty) nie mają związku z asystentem głosowym. Bez nagrań (`Nagrania: nie`) plik jest pomijany.
- **Status**: ✅

### A3: Jeden klient (jedna aplikacja) — bez architektury multi-client/BFF i bez faz z suffixami
- **Category**: architektura
- **Source**: brak — spec nie wspomina o wielu klientach
- **Rationale**: Nic nie wskazuje na web + mobile; standardowa numeracja faz 1, 2, 3…
- **Status**: ✅

### A4: Brak fazy design system / UI z mockupów
- **Category**: wymagania
- **Source**: telesma/spec/ux_ui/LINKS.md:1 (pusty szablon), brak plików mockupów
- **Rationale**: Brak mockupów → brak zadań `(ui-tokens/ui-component/ui-screen)`; ewentualny interfejs (Q10) projektowany minimalistycznie.
- **Status**: ✅

### A5: Workspace Cargo podzielony na crate'y wg odpowiedzialności (audio, stt, core/pipeline, app/bin)
- **Category**: architektura
- **Source**: konwencja stack-u (Rust)
- **Rationale**: Izoluje kod zależny od GPU (stt) od reszty, ułatwia testowanie logiki bez GPU i szybszą kompilację.
- **Status**: ✅

### A6: Narzędzia jakości: `cargo test` (testy), `cargo clippy -- -D warnings` + `cargo fmt --check` (linter)
- **Category**: technologia
- **Source**: konwencja stack-u (Rust); config.md ma niewypełnione pola testów/lintera
- **Rationale**: Standard ekosystemu Rust; zostanie wpisany do `telesma/config.md` przy zadaniu 1.1.
- **Status**: ✅

### A7: Obsługa błędów: `thiserror` w bibliotekach, `anyhow` w binarce; logowanie przez `tracing`
- **Category**: technologia
- **Source**: konwencja stack-u (Rust)
- **Rationale**: De-facto standard w Rust; strukturalne logi przydatne przy diagnozie latencji pipeline'u.
- **Status**: ✅

### A8: Przechwytywanie audio przez `cpal`, normalizacja do 16 kHz mono f32 (wymagane przez Whisper)
- **Category**: technologia
- **Source**: standard branżowy (Whisper przyjmuje 16 kHz mono)
- **Rationale**: `cpal` to wieloplatformowy standard audio w Rust; resampling np. `rubato`.
- **Status**: ✅

### A9: Konfiguracja w pliku TOML (ścieżka modelu, urządzenie audio, język, skróty) + nadpisanie flagami CLI
- **Category**: technologia
- **Source**: konwencja stack-u (Rust: `serde` + `toml` + `clap`)
- **Rationale**: Brak wymagań co do konfiguracji; TOML jest idiomatyczny dla Rust.
- **Status**: ✅

### A10: Brak GPU / brak wymaganego backendu GPU = czytelny błąd przy starcie i zakończenie (bez fallbacku na CPU)
- **Category**: wymagania
- **Source**: WYTYCZNE_TECHNICZNE.md:70 — „model ma działać tylko na GPU"
- **Rationale**: Dosłowna interpretacja ograniczenia; cichy fallback na CPU dawałby nieakceptowalną latencję i maskował błąd konfiguracji.
- **Status**: ✅

### A11: Testy STT oparte o fixtures WAV z oczekiwaną transkrypcją (porównanie przez WER / zawieranie słów kluczowych); testy wymagające GPU oznaczone `#[ignore]` lub feature flagą i uruchamiane lokalnie
- **Category**: technologia
- **Source**: brak — luka w strategii testów
- **Rationale**: Logika pipeline'u testowana z mockiem STT (bez GPU); integracja z modelem — na maszynie z GPU.
- **Status**: ✅

### A12: Domyślnie brak trwałego zapisu nagrań audio (przetwarzanie w pamięci); transkrypcje tylko w logu na poziomie debug
- **Category**: wymagania
- **Source**: standard branżowy (privacy by default)
- **Rationale**: Asystent głosowy przetwarza wrażliwe dane; zapis może być opcją w konfiguracji.
- **Status**: ✅

## Questions

### Q1: Jaki jest główny cel asystenta — co user osiąga głosem?
- **Category**: wymagania
- **Source**: brak — spec nie zawiera żadnych wymagań funkcjonalnych (WYTYCZNE_TECHNICZNE.md:74-86 odsyła do nieistniejących plików)
- **Proposed answer**: Lokalny asystent desktopowy: user mówi → transkrypcja Whisper → interpretacja przez LLM → wykonanie prostych akcji / odpowiedź. Alternatywy: (a) samo dyktowanie tekstu do aktywnego okna, (b) komendy systemowe z ustaloną listą, (c) konwersacja z LLM, (d) kombinacja. MVP proponuję: (a) + (c).
- **Answer**: 
- **Status**: ✅

### Q2: Na jakim systemie operacyjnym i jakim GPU ma działać aplikacja?
- **Category**: ograniczenia
- **Source**: WYTYCZNE_TECHNICZNE.md:34, 70 (puste środowisko docelowe; „tylko GPU")
- **Proposed answer**: Linux x86_64 z GPU NVIDIA (CUDA 12). Jeśli docelowo macOS (Apple Silicon) — backend Metal; jeśli oba — feature flagi `cuda`/`metal`.
- **Answer**: 
- **Status**: ✅

### Q3: Jakiego runtime'u inferencji Whisper użyć z Rust?
- **Category**: technologia
- **Source**: WYTYCZNE_TECHNICZNE.md:24, 30
- **Proposed answer**: `whisper-rs` (bindingi do whisper.cpp) z backendem CUDA/Metal i modelem GGML large-v3-turbo — dojrzałe, szybkie, dobre wsparcie GPU. Alternatywa: `candle` (czysty Rust, mniej dojrzała wydajność) lub CTranslate2 przez FFI.
- **Answer**: 
- **Status**: ✅

### Q4: W jakich językach asystent ma rozpoznawać mowę?
- **Category**: wymagania
- **Source**: WYTYCZNE_TECHNICZNE.md:61 (Lokalizacja / i18n — puste)
- **Proposed answer**: Polski jako domyślny, z możliwością ustawienia angielskiego lub autodetekcji języka w konfiguracji.
- **Answer**: 
- **Status**: ✅

### Q5: Jak user aktywuje nasłuch?
- **Category**: wymagania
- **Source**: brak — nie opisano aktywacji
- **Proposed answer**: MVP: globalny skrót klawiszowy push-to-talk (przytrzymaj = nagrywaj) lub toggle. Wake word („Hej asystencie") jako późniejsza faza — wymaga osobnego modelu (np. openWakeWord/Porcupine).
- **Answer**: 
- **Status**: ✅

### Q6: Co dzieje się z tekstem po transkrypcji (wyjście)?
- **Category**: wymagania
- **Source**: brak — nie opisano wyjścia
- **Proposed answer**: Zależnie od Q1: (a) wpisanie tekstu do aktywnego okna (symulacja klawiatury, np. `enigo`) i/lub schowek, (b) przekazanie do LLM i wyświetlenie/odczytanie odpowiedzi.
- **Answer**: 
- **Status**: ✅

### Q7: Czy asystent korzysta z LLM do interpretacji/odpowiedzi — a jeśli tak, lokalnego czy chmurowego?
- **Category**: architektura
- **Source**: WYTYCZNE_TECHNICZNE.md:40-50 (integracje zewnętrzne — puste)
- **Proposed answer**: Abstrakcja `LlmProvider` z jedną implementacją w MVP. Propozycja: lokalny model przez API zgodne z OpenAI (Ollama/llama.cpp server) — spójne z „tylko GPU" i prywatnością; opcjonalnie Claude API jako druga implementacja.
- **Answer**: 
- **Status**: ✅

### Q8: Czy asystent ma odpowiadać głosem (TTS)? Jeśli tak — jakim silnikiem?
- **Category**: wymagania
- **Source**: brak — nie opisano wyjścia głosowego
- **Proposed answer**: MVP bez TTS (odpowiedź tekstowa). W kolejnej fazie lokalny TTS (np. Piper — ma polskie głosy) przez proces zewnętrzny lub bindingi.
- **Answer**: 
- **Status**: ✅

### Q9: Transkrypcja po zakończeniu wypowiedzi czy strumieniowo (częściowe wyniki w trakcie mówienia)?
- **Category**: wymagania
- **Source**: WYTYCZNE_TECHNICZNE.md:56 (Wydajność — puste)
- **Proposed answer**: MVP: transkrypcja całej wypowiedzi po zwolnieniu klawisza / wykryciu ciszy przez VAD (prostsze, dokładniejsze). Strumieniowanie z częściowymi wynikami jako rozszerzenie.
- **Answer**: 
- **Status**: ✅

### Q10: Jaki interfejs ma mieć aplikacja?
- **Category**: wymagania
- **Source**: brak — brak mockupów (telesma/spec/ux_ui/LINKS.md pusty) i opisu UI
- **Proposed answer**: Demon/proces działający w tle sterowany CLI (`voice-asystent run`, `devices`, `test-mic`) + minimalny wskaźnik stanu w zasobniku systemowym (tray: nasłuch/przetwarzanie/błąd). Bez pełnego GUI w MVP.
- **Answer**: 
- **Status**: ✅

### Q11: Jaki jest docelowy budżet latencji (koniec mowy → wynik)?
- **Category**: ograniczenia
- **Source**: WYTYCZNE_TECHNICZNE.md:56 (puste)
- **Proposed answer**: p95 < 1 s od końca wypowiedzi do gotowej transkrypcji dla wypowiedzi do 15 s na GPU klasy RTX 3060+; z LLM dodatkowo < 3 s do pierwszego tokenu odpowiedzi.
- **Answer**: 
- **Status**: ✅

### Q12: Jakie jest minimalne wspierane GPU / VRAM?
- **Category**: ograniczenia
- **Source**: WYTYCZNE_TECHNICZNE.md:70
- **Proposed answer**: ≥ 6 GB VRAM (large-v3-turbo w fp16 ~1.6 GB + zapas na lokalny LLM, jeśli Q7 = lokalny — wtedy realnie ≥ 12 GB). Sprawdzanie VRAM przy starcie z czytelnym komunikatem.
- **Answer**: 
- **Status**: ✅

### Q13: Czy aplikacja ma działać w pełni offline?
- **Category**: ograniczenia
- **Source**: WYTYCZNE_TECHNICZNE.md:59, 64 (compliance, offline — puste)
- **Proposed answer**: Tak — STT zawsze lokalnie; jedyny dopuszczalny ruch sieciowy to jednorazowe pobranie modelu i ewentualnie chmurowy LLM, jeśli zostanie wybrany w Q7.
- **Answer**: 
- **Status**: ✅

### Q14: Jak model Whisper trafia na maszynę użytkownika?
- **Category**: technologia
- **Source**: brak — nie opisano dystrybucji modelu
- **Proposed answer**: Komenda `voice-asystent model download` pobiera plik modelu z Hugging Face do katalogu cache (`~/.cache/voice-asystent/`) z weryfikacją sumy SHA-256; ścieżkę można nadpisać w konfiguracji. Model nie jest w repo.
- **Answer**: 
- **Status**: ✅

### Q15: Jak aplikacja ma być dystrybuowana i uruchamiana?
- **Category**: ograniczenia
- **Source**: WYTYCZNE_TECHNICZNE.md:34-36 (środowisko, kontenery, CI/CD — puste)
- **Proposed answer**: Pojedyncza binarka budowana `cargo build --release` (z feature flagą backendu GPU), uruchamiana lokalnie; bez Dockera (dostęp do mikrofonu i GPU z kontenera komplikuje setup). Bez CI w MVP lub CI tylko dla `clippy` + testów bez GPU.
- **Answer**: 
- **Status**: ✅

### Q16: Czy potrzebne jest VAD (wykrywanie mowy/ciszy) i wybór urządzenia wejściowego?
- **Category**: wymagania
- **Source**: brak — nie opisano obsługi audio
- **Proposed answer**: Tak: wybór mikrofonu w konfiguracji (domyślnie systemowy) i VAD (np. Silero VAD lub energetyczny) do przycinania ciszy i automatycznego zakończenia nagrania w trybie toggle.
- **Answer**: 
- **Status**: ✅

### Q17: Jeśli asystent wykonuje akcje — jakie konkretnie w MVP?
- **Category**: priorytety
- **Source**: brak — nie opisano komend/akcji
- **Proposed answer**: Jeśli Q1 obejmuje komendy: zamknięta lista akcji w MVP (uruchom aplikację, otwórz URL, wpisz tekst, odczytaj godzinę/datę), definiowana w konfiguracji; bez dowolnego wykonywania poleceń powłoki (bezpieczeństwo). Jeśli Q1 = tylko dyktowanie — brak akcji.
- **Answer**: 
- **Status**: ✅

### Q18: Jaki jest zakres MVP i kolejność rozszerzeń?
- **Category**: priorytety
- **Source**: brak — brak priorytetów w spec
- **Proposed answer**: MVP: push-to-talk → nagranie → Whisper large-v3-turbo na GPU → tekst wpisany do aktywnego okna / schowek + CLI i konfiguracja. Faza 2: LLM. Faza 3: TTS. Faza 4: wake word, streaming, tray.
- **Answer**: 
- **Status**: ✅
