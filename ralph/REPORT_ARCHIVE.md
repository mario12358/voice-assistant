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
