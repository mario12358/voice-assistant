# ralph/QUESTIONS.md - Doprecyzowanie projektu

## Status

- **Stan**: ukończony
- **Pytań łącznie**: 18
- **Odpowiedzianych**: 18
- **Założeń**: 12
- **Sprzeczności**: 0
- **Data generowania**: 2026-10-08

## Przeanalizowane dokumenty

- `spec/WYTYCZNE_TECHNICZNE.md` (86 linii) — jedyne źródło merytoryczne: Rust, model large-v3-turbo, tylko GPU
- `spec/APP_FLOW.md` (101 linii) — niewypełniony szablon (marker `RALPH_APP_FLOW_TEMPLATE`)
- `spec/ux_ui/LINKS.md` (28 linii) — niewypełniony szablon (marker `RALPH_UX_LINKS_TEMPLATE`)

Ocena jakości: kompletność ~5%, jednoznaczność niska → duża liczba pytań (szczegóły w `ralph/SPEC_INDEX.md`).

## Sprzeczności w dokumentacji

<!-- Brak wykrytych sprzeczności -->

## Założenia

### A1: Model „large-v3-turbo" to OpenAI Whisper large-v3-turbo (rozpoznawanie mowy, STT)
- **Kategoria**: technologia
- **Źródło**: WYTYCZNE_TECHNICZNE.md:30
- **Uzasadnienie**: To jedyny powszechnie znany model o tej nazwie; w połączeniu z nazwą projektu „voice_asystent" oznacza warstwę speech-to-text.
- **Status**: ✅

### A2: Szablonowy `spec/APP_FLOW.md` (rejestracja, projekty web) NIE jest źródłem wymagań
- **Kategoria**: wymagania
- **Źródło**: APP_FLOW.md:1 (marker szablonu), APP_FLOW.md:36-81
- **Uzasadnienie**: Plik ma marker szablonu, a przykładowe flow (konta, projekty) nie mają związku z asystentem głosowym. Bez nagrań (`Nagrania: nie`) plik jest pomijany.
- **Status**: ✅

### A3: Jeden klient (jedna aplikacja) — bez architektury multi-client/BFF i bez faz z suffixami
- **Kategoria**: architektura
- **Źródło**: brak — spec nie wspomina o wielu klientach
- **Uzasadnienie**: Nic nie wskazuje na web + mobile; standardowa numeracja faz 1, 2, 3…
- **Status**: ✅

### A4: Brak fazy design system / UI z mockupów
- **Kategoria**: wymagania
- **Źródło**: spec/ux_ui/LINKS.md:1 (pusty szablon), brak plików mockupów
- **Uzasadnienie**: Brak mockupów → brak zadań `(ui-tokens/ui-component/ui-screen)`; ewentualny interfejs (Q10) projektowany minimalistycznie.
- **Status**: ✅

### A5: Workspace Cargo podzielony na crate'y wg odpowiedzialności (audio, stt, core/pipeline, app/bin)
- **Kategoria**: architektura
- **Źródło**: konwencja stack-u (Rust)
- **Uzasadnienie**: Izoluje kod zależny od GPU (stt) od reszty, ułatwia testowanie logiki bez GPU i szybszą kompilację.
- **Status**: ✅

### A6: Narzędzia jakości: `cargo test` (testy), `cargo clippy -- -D warnings` + `cargo fmt --check` (linter)
- **Kategoria**: technologia
- **Źródło**: konwencja stack-u (Rust); config.md ma niewypełnione pola testów/lintera
- **Uzasadnienie**: Standard ekosystemu Rust; zostanie wpisany do `ralph/config.md` przy zadaniu 1.1.
- **Status**: ✅

### A7: Obsługa błędów: `thiserror` w bibliotekach, `anyhow` w binarce; logowanie przez `tracing`
- **Kategoria**: technologia
- **Źródło**: konwencja stack-u (Rust)
- **Uzasadnienie**: De-facto standard w Rust; strukturalne logi przydatne przy diagnozie latencji pipeline'u.
- **Status**: ✅

### A8: Przechwytywanie audio przez `cpal`, normalizacja do 16 kHz mono f32 (wymagane przez Whisper)
- **Kategoria**: technologia
- **Źródło**: standard branżowy (Whisper przyjmuje 16 kHz mono)
- **Uzasadnienie**: `cpal` to wieloplatformowy standard audio w Rust; resampling np. `rubato`.
- **Status**: ✅

### A9: Konfiguracja w pliku TOML (ścieżka modelu, urządzenie audio, język, skróty) + nadpisanie flagami CLI
- **Kategoria**: technologia
- **Źródło**: konwencja stack-u (Rust: `serde` + `toml` + `clap`)
- **Uzasadnienie**: Brak wymagań co do konfiguracji; TOML jest idiomatyczny dla Rust.
- **Status**: ✅

### A10: Brak GPU / brak wymaganego backendu GPU = czytelny błąd przy starcie i zakończenie (bez fallbacku na CPU)
- **Kategoria**: wymagania
- **Źródło**: WYTYCZNE_TECHNICZNE.md:70 — „model ma działać tylko na GPU"
- **Uzasadnienie**: Dosłowna interpretacja ograniczenia; cichy fallback na CPU dawałby nieakceptowalną latencję i maskował błąd konfiguracji.
- **Status**: ✅

### A11: Testy STT oparte o fixtures WAV z oczekiwaną transkrypcją (porównanie przez WER / zawieranie słów kluczowych); testy wymagające GPU oznaczone `#[ignore]` lub feature flagą i uruchamiane lokalnie
- **Kategoria**: technologia
- **Źródło**: brak — luka w strategii testów
- **Uzasadnienie**: Logika pipeline'u testowana z mockiem STT (bez GPU); integracja z modelem — na maszynie z GPU.
- **Status**: ✅

### A12: Domyślnie brak trwałego zapisu nagrań audio (przetwarzanie w pamięci); transkrypcje tylko w logu na poziomie debug
- **Kategoria**: wymagania
- **Źródło**: standard branżowy (privacy by default)
- **Uzasadnienie**: Asystent głosowy przetwarza wrażliwe dane; zapis może być opcją w konfiguracji.
- **Status**: ✅

## Pytania

### Q1: Jaki jest główny cel asystenta — co user osiąga głosem?
- **Kategoria**: wymagania
- **Źródło**: brak — spec nie zawiera żadnych wymagań funkcjonalnych (WYTYCZNE_TECHNICZNE.md:74-86 odsyła do nieistniejących plików)
- **Proponowana odpowiedź**: Lokalny asystent desktopowy: user mówi → transkrypcja Whisper → interpretacja przez LLM → wykonanie prostych akcji / odpowiedź. Alternatywy: (a) samo dyktowanie tekstu do aktywnego okna, (b) komendy systemowe z ustaloną listą, (c) konwersacja z LLM, (d) kombinacja. MVP proponuję: (a) + (c).
- **Odpowiedź**: 
- **Status**: ✅

### Q2: Na jakim systemie operacyjnym i jakim GPU ma działać aplikacja?
- **Kategoria**: ograniczenia
- **Źródło**: WYTYCZNE_TECHNICZNE.md:34, 70 (puste środowisko docelowe; „tylko GPU")
- **Proponowana odpowiedź**: Linux x86_64 z GPU NVIDIA (CUDA 12). Jeśli docelowo macOS (Apple Silicon) — backend Metal; jeśli oba — feature flagi `cuda`/`metal`.
- **Odpowiedź**: 
- **Status**: ✅

### Q3: Jakiego runtime'u inferencji Whisper użyć z Rust?
- **Kategoria**: technologia
- **Źródło**: WYTYCZNE_TECHNICZNE.md:24, 30
- **Proponowana odpowiedź**: `whisper-rs` (bindingi do whisper.cpp) z backendem CUDA/Metal i modelem GGML large-v3-turbo — dojrzałe, szybkie, dobre wsparcie GPU. Alternatywa: `candle` (czysty Rust, mniej dojrzała wydajność) lub CTranslate2 przez FFI.
- **Odpowiedź**: 
- **Status**: ✅

### Q4: W jakich językach asystent ma rozpoznawać mowę?
- **Kategoria**: wymagania
- **Źródło**: WYTYCZNE_TECHNICZNE.md:61 (Lokalizacja / i18n — puste)
- **Proponowana odpowiedź**: Polski jako domyślny, z możliwością ustawienia angielskiego lub autodetekcji języka w konfiguracji.
- **Odpowiedź**: 
- **Status**: ✅

### Q5: Jak user aktywuje nasłuch?
- **Kategoria**: wymagania
- **Źródło**: brak — nie opisano aktywacji
- **Proponowana odpowiedź**: MVP: globalny skrót klawiszowy push-to-talk (przytrzymaj = nagrywaj) lub toggle. Wake word („Hej asystencie") jako późniejsza faza — wymaga osobnego modelu (np. openWakeWord/Porcupine).
- **Odpowiedź**: 
- **Status**: ✅

### Q6: Co dzieje się z tekstem po transkrypcji (wyjście)?
- **Kategoria**: wymagania
- **Źródło**: brak — nie opisano wyjścia
- **Proponowana odpowiedź**: Zależnie od Q1: (a) wpisanie tekstu do aktywnego okna (symulacja klawiatury, np. `enigo`) i/lub schowek, (b) przekazanie do LLM i wyświetlenie/odczytanie odpowiedzi.
- **Odpowiedź**: 
- **Status**: ✅

### Q7: Czy asystent korzysta z LLM do interpretacji/odpowiedzi — a jeśli tak, lokalnego czy chmurowego?
- **Kategoria**: architektura
- **Źródło**: WYTYCZNE_TECHNICZNE.md:40-50 (integracje zewnętrzne — puste)
- **Proponowana odpowiedź**: Abstrakcja `LlmProvider` z jedną implementacją w MVP. Propozycja: lokalny model przez API zgodne z OpenAI (Ollama/llama.cpp server) — spójne z „tylko GPU" i prywatnością; opcjonalnie Claude API jako druga implementacja.
- **Odpowiedź**: 
- **Status**: ✅

### Q8: Czy asystent ma odpowiadać głosem (TTS)? Jeśli tak — jakim silnikiem?
- **Kategoria**: wymagania
- **Źródło**: brak — nie opisano wyjścia głosowego
- **Proponowana odpowiedź**: MVP bez TTS (odpowiedź tekstowa). W kolejnej fazie lokalny TTS (np. Piper — ma polskie głosy) przez proces zewnętrzny lub bindingi.
- **Odpowiedź**: 
- **Status**: ✅

### Q9: Transkrypcja po zakończeniu wypowiedzi czy strumieniowo (częściowe wyniki w trakcie mówienia)?
- **Kategoria**: wymagania
- **Źródło**: WYTYCZNE_TECHNICZNE.md:56 (Wydajność — puste)
- **Proponowana odpowiedź**: MVP: transkrypcja całej wypowiedzi po zwolnieniu klawisza / wykryciu ciszy przez VAD (prostsze, dokładniejsze). Strumieniowanie z częściowymi wynikami jako rozszerzenie.
- **Odpowiedź**: 
- **Status**: ✅

### Q10: Jaki interfejs ma mieć aplikacja?
- **Kategoria**: wymagania
- **Źródło**: brak — brak mockupów (spec/ux_ui/LINKS.md pusty) i opisu UI
- **Proponowana odpowiedź**: Demon/proces działający w tle sterowany CLI (`voice-asystent run`, `devices`, `test-mic`) + minimalny wskaźnik stanu w zasobniku systemowym (tray: nasłuch/przetwarzanie/błąd). Bez pełnego GUI w MVP.
- **Odpowiedź**: 
- **Status**: ✅

### Q11: Jaki jest docelowy budżet latencji (koniec mowy → wynik)?
- **Kategoria**: ograniczenia
- **Źródło**: WYTYCZNE_TECHNICZNE.md:56 (puste)
- **Proponowana odpowiedź**: p95 < 1 s od końca wypowiedzi do gotowej transkrypcji dla wypowiedzi do 15 s na GPU klasy RTX 3060+; z LLM dodatkowo < 3 s do pierwszego tokenu odpowiedzi.
- **Odpowiedź**: 
- **Status**: ✅

### Q12: Jakie jest minimalne wspierane GPU / VRAM?
- **Kategoria**: ograniczenia
- **Źródło**: WYTYCZNE_TECHNICZNE.md:70
- **Proponowana odpowiedź**: ≥ 6 GB VRAM (large-v3-turbo w fp16 ~1.6 GB + zapas na lokalny LLM, jeśli Q7 = lokalny — wtedy realnie ≥ 12 GB). Sprawdzanie VRAM przy starcie z czytelnym komunikatem.
- **Odpowiedź**: 
- **Status**: ✅

### Q13: Czy aplikacja ma działać w pełni offline?
- **Kategoria**: ograniczenia
- **Źródło**: WYTYCZNE_TECHNICZNE.md:59, 64 (compliance, offline — puste)
- **Proponowana odpowiedź**: Tak — STT zawsze lokalnie; jedyny dopuszczalny ruch sieciowy to jednorazowe pobranie modelu i ewentualnie chmurowy LLM, jeśli zostanie wybrany w Q7.
- **Odpowiedź**: 
- **Status**: ✅

### Q14: Jak model Whisper trafia na maszynę użytkownika?
- **Kategoria**: technologia
- **Źródło**: brak — nie opisano dystrybucji modelu
- **Proponowana odpowiedź**: Komenda `voice-asystent model download` pobiera plik modelu z Hugging Face do katalogu cache (`~/.cache/voice-asystent/`) z weryfikacją sumy SHA-256; ścieżkę można nadpisać w konfiguracji. Model nie jest w repo.
- **Odpowiedź**: 
- **Status**: ✅

### Q15: Jak aplikacja ma być dystrybuowana i uruchamiana?
- **Kategoria**: ograniczenia
- **Źródło**: WYTYCZNE_TECHNICZNE.md:34-36 (środowisko, kontenery, CI/CD — puste)
- **Proponowana odpowiedź**: Pojedyncza binarka budowana `cargo build --release` (z feature flagą backendu GPU), uruchamiana lokalnie; bez Dockera (dostęp do mikrofonu i GPU z kontenera komplikuje setup). Bez CI w MVP lub CI tylko dla `clippy` + testów bez GPU.
- **Odpowiedź**: 
- **Status**: ✅

### Q16: Czy potrzebne jest VAD (wykrywanie mowy/ciszy) i wybór urządzenia wejściowego?
- **Kategoria**: wymagania
- **Źródło**: brak — nie opisano obsługi audio
- **Proponowana odpowiedź**: Tak: wybór mikrofonu w konfiguracji (domyślnie systemowy) i VAD (np. Silero VAD lub energetyczny) do przycinania ciszy i automatycznego zakończenia nagrania w trybie toggle.
- **Odpowiedź**: 
- **Status**: ✅

### Q17: Jeśli asystent wykonuje akcje — jakie konkretnie w MVP?
- **Kategoria**: priorytety
- **Źródło**: brak — nie opisano komend/akcji
- **Proponowana odpowiedź**: Jeśli Q1 obejmuje komendy: zamknięta lista akcji w MVP (uruchom aplikację, otwórz URL, wpisz tekst, odczytaj godzinę/datę), definiowana w konfiguracji; bez dowolnego wykonywania poleceń powłoki (bezpieczeństwo). Jeśli Q1 = tylko dyktowanie — brak akcji.
- **Odpowiedź**: 
- **Status**: ✅

### Q18: Jaki jest zakres MVP i kolejność rozszerzeń?
- **Kategoria**: priorytety
- **Źródło**: brak — brak priorytetów w spec
- **Proponowana odpowiedź**: MVP: push-to-talk → nagranie → Whisper large-v3-turbo na GPU → tekst wpisany do aktywnego okna / schowek + CLI i konfiguracja. Faza 2: LLM. Faza 3: TTS. Faza 4: wake word, streaming, tray.
- **Odpowiedź**: 
- **Status**: ✅
