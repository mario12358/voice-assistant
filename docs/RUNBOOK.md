# RUNBOOK — VoiceAsystent

Instrukcja dla osoby przejmującej projekt: jak zbudować, uruchomić, przetestować i wydać
aplikację oraz gdzie szukać, gdy coś nie działa. Opis dla użytkownika końcowego jest
w [README.md](../README.md).

## Platforma i narzędzia

- **Tylko macOS na Apple Silicon.** Workspace ma `compile_error!` poza macOS; GPU Metal jest
  wymagane w czasie działania (bez niego aplikacja startuje, ale nagrywanie jest zablokowane).
- Xcode Command Line Tools (`xcode-select --install`) — kompilator C/C++ i Metal dla whisper.cpp,
  `codesign`, `iconutil`, `plutil`, `hdiutil`.
- Rust stable (`rustup`), minimum 1.85, edycja 2024; komponenty `clippy` i `rustfmt`.
- `python3` (biblioteka standardowa) — generowanie ikony aplikacji.
- Do CI dodatkowo `cargo-nextest` (raport JUnit dla Specky).

## Zmienne środowiskowe

Aplikacja nie potrzebuje żadnych sekretów ani kluczy.

| Zmienna | Gdzie | Znaczenie |
|---------|-------|-----------|
| `RUST_LOG` | aplikacja, `va-dev` | filtr logów `tracing` (np. `RUST_LOG=debug`); ma pierwszeństwo przed `-v` |
| `HOME` | testy | testy startu ustawiają tymczasowy `HOME`, żeby nie dotykać prawdziwej konfiguracji i modelu |
| `CARGO_TARGET_DIR` | opcjonalnie | osobny katalog buildu (np. przy mutacjach testów) — pilnuj miejsca na dysku |

## Ścieżki w systemie użytkownika

| Co | Ścieżka |
|----|---------|
| Konfiguracja | `~/Library/Application Support/VoiceAsystent/config.toml` |
| Model | `~/Library/Application Support/VoiceAsystent/models/ggml-large-v3-turbo.bin` (1 624 555 275 B, SHA-256 `1fc70f77…e2bc69`); częściowe pobranie: `.part` obok |
| Logi | `~/Library/Logs/VoiceAsystent/voice-asystent.log.*` (JSON, dziennie) + tekst na stderr |

Portów sieciowych, bazy danych, migracji ani kont testowych nie ma. Jedyne połączenie
sieciowe to pobranie modelu z `https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo.bin`.

## Komendy

```bash
export PATH="$HOME/.cargo/bin:$PATH"

# Build i uruchomienie z paska menu (debug; model i GPU jak w wydaniu)
cargo run -p voice-asystent
cargo run -p voice-asystent -- --self-check   # tylko sprawdzenia startowe (GPU, model), bez GUI

# Narzędzia deweloperskie
cargo run -p va-dev -- config                 # obowiązująca konfiguracja
cargo run -p va-dev -- devices                # mikrofony (* domyślny, > z konfiguracji)
cargo run -p va-dev -- mic-test --seconds 5   # poziom sygnału; --save plik.wav zapisuje 16 kHz mono
cargo run -p va-dev -- model-download         # pobiera/wznawia model, sprawdza SHA-256
cargo run -p va-dev -- transcribe plik.wav    # transkrypcja na Metalu, tekst na stdout

# Testy
cargo test --workspace                                  # szybkie (bez mikrofonu/GPU/modelu)
cargo test --workspace -- --include-ignored             # pełne: mikrofon, Metal, model, schowek, build release
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check

# Wydanie
scripts/build-app.sh                                    # dist/VoiceAsystent.app
scripts/build-dmg.sh                                    # dist/VoiceAsystent-<wersja>.dmg
```

Opcje skryptów: `--out <katalog>`, `--version <v1.2.3>`; `build-app.sh --binary <plik>` składa
bundle z gotowej binarki (tak robią testy w CI), `build-dmg.sh --app <bundle>` pakuje gotowy
bundle. Wersja domyślnie z `git describe --tags --match 'v*'`.

## Struktura

| Crate | Rola |
|-------|------|
| `crates/config` | `Config` (TOML) i ścieżki macOS |
| `crates/audio` | mikrofony, nagrywanie (cpal) → 16 kHz mono, przycinanie ciszy |
| `crates/model` | pobieranie modelu (HTTP Range, wznawianie, SHA-256) |
| `crates/stt` | whisper-rs z feature `metal`, sprawdzenie GPU |
| `crates/clipboard` | schowek (arboard); pusta transkrypcja nie zmienia schowka |
| `crates/core` | automat stanów i kontroler (bez GUI), logowanie |
| `apps/voice-asystent` | aplikacja paska menu (tao, tray-icon, global-hotkey) |
| `apps/va-dev` | CLI deweloperskie |
| `scripts/` | `build-app.sh`, `build-dmg.sh`, `app-icon.py` |

## Testy — co wymaga sprzętu

Testy oznaczone `#[ignore = "wymaga …"]` potrzebują mikrofonu, GPU Metal, pobranego modelu,
prawdziwego schowka albo builda release (kilka minut). CI (`.github/workflows/specky-ci.yml`,
`macos-latest`) uruchamia tylko testy bez `#[ignore]`; pełny zestaw uruchamiaj lokalnie przed
wydaniem. Testy bundla i obrazu (`apps/voice-asystent/tests/bundle.rs`, `dmg.rs`) składają
aplikację z binarki debug i montują obraz przez `hdiutil attach -nobrowse`.

## Wydanie wersji

1. Pełny zestaw testów zielony, drzewo czyste (`git status --porcelain` puste).
2. Scenariusze ręczne: `docs/test-scenarios/vX.Y.Z.md` (szablon `ralph/TEST_SCENARIOS_TEMPLATE.md`).
3. `git tag -a vX.Y.Z -m "…"`, `git push origin vX.Y.Z` (tagi nie idą ze zwykłym `git push`).
4. `scripts/build-dmg.sh` na otagowanym commicie → `dist/VoiceAsystent-X.Y.Z.dmg`.
   Wersja w Info.plist i w nazwie obrazu pochodzi z `git describe`; w repo nie ma pola wersji
   do podbijania (`version` w `Cargo.toml` jest stałe).
5. Aplikacja jest podpisana ad-hoc, bez notaryzacji — odbiorca obrazu z sieci przechodzi przez
   Gatekeeper („Otwórz mimo to"), patrz README.

## Rozwiązywanie problemów

| Objaw | Przyczyna i co zrobić |
|-------|----------------------|
| Menu: „Wymagane GPU (Metal) — nagrywanie niedostępne" | brak urządzenia Metal (Intel/VM). Aplikacja celowo nie przełącza się na CPU. Sprawdź: `cargo run -p voice-asystent -- --self-check` i log `GPU Metal dostępne` / `Wymagane GPU (Metal)` |
| Menu: „Brak modelu — nagrywanie niedostępne" / „Pobieranie modelu nieudane: …" | wybierz **Ponów pobieranie** w menu albo `cargo run -p va-dev -- model-download`. Uszkodzony plik (zła suma) jest usuwany automatycznie; ręcznie: usuń `models/ggml-large-v3-turbo.bin` i `.part` |
| Powiadomienie „Brak dostępu do mikrofonu" | nagranie dało same zera: brak zgody (Ustawienia → Prywatność i ochrona → Mikrofon) albo wirtualne wejście bez sygnału. Diagnoza: `cargo run -p va-dev -- mic-test` |
| Skrót nie działa, w menu komunikat o konflikcie | kombinacja `ctrl+cmd+r`/`ctrl+cmd+s` zajęta przez inną aplikację; zwolnij ją tam albo używaj kliknięcia ikony |
| Nagranie skończyło się samo, powiadomienie „Osiągnięto limit długości nagrania" | limit `max_recording_secs` (domyślnie 600 s): materiał do limitu jest transkrybowany jak po Stop; dłuższe dyktowanie → podnieś wartość w `config.toml` |
| Transkrypcja pusta, schowek bez zmian | nagranie było ciszą poniżej `silence.threshold_rms` (domyślnie 0.01) — obniż próg w `config.toml` albo sprawdź mikrofon `mic-test` |
| Start aplikacji trwa kilka sekund | przy każdym starcie liczona jest suma SHA-256 modelu (1,6 GB, ok. 3–4 s) i ładowany model na GPU |
| Powiadomienia pokazują się jako „Terminal" | aplikacja uruchomiona z `cargo run`, nie z bundla; z `dist/VoiceAsystent.app` pokazują się pod nazwą VoiceAsystent |
| `cargo` nie znalezione w nowym shellu | `export PATH="$HOME/.cargo/bin:$PATH"` |
| Brak miejsca na dysku przy testach | artefakty w `target/` (debug + release + whisper.cpp) zajmują kilka GB; `cargo clean` |
| Obraz .dmg nie montuje się w teście („Resource busy") | poprzedni test nie odmontował wolumenu: `hdiutil info`, potem `hdiutil detach <mountpoint>` |
