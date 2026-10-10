---
data: 2026-10-10
rodzaj: rozwój
priorytet: normalny
dotyczy: ogólne (nowe wymaganie Specky, sekcja „Rozpoznawanie mowy”; kod: crates/model, va-config, startup.rs, model_menu.rs, app.rs)
specky_req: 01M4KD15KW259CT135K1JBKCWR
specky_hash: e7c6470
---

# VA-MODEL-4 — wariant skwantyzowany large-v3-turbo q5_0 do wyboru w podmenu „Model”

Zaakceptowane 2026-10-10 (wsad 01M4KC9EX5DBY9R1XYWNVHGX4Z), powód z kolejki: `to_build`.

**Treść (v1 @e7c6470):** Oprócz pełnego modelu large-v3-turbo (1,6 GB) dostępny jest jego wariant
skwantyzowany q5_0 (ok. 0,6 GB) do wyboru w podmenu „Model”; przełączenie pobiera wybrany wariant
(z sumą SHA-256 i wznawianiem) i przeładowuje model bez restartu, a niewybrany wariant można usunąć.

**Kryteria:**
1. (crit: 01M4KD15M5E90XXM60PPS2S33J) Pozycje „Pełny (1,6 GB)” i „Skwantyzowany q5_0 (0,6 GB)” z zaznaczonym używanym; linia stanu z nazwą wariantu
2. (crit: 01M4KD15M6BESGW2M37ZY4WV7S) Wybór niepobranego wariantu → pobieranie z postępem → przeładowanie bez restartu; wybór w config.toml
3. (crit: 01M4KD15M64AGF6T592KW5WQS7) Oba warianty z Hugging Face ggerganov/whisper.cpp, SHA-256, wznawianie
4. (crit: 01M4KD15M63SWHPZAMN9BYDK0W) „Usuń model” usuwa tylko wybrany wariant; drugi, jeśli pobrany, osobną pozycją
5. (crit: 01M4KD15M64RDY5XTBQV6MK4FK) `speech_pl.wav` wariantem q5_0 ma te same słowa kluczowe co pełnym

**Kontekst:** VA-STT-1 bez zmian (q5_0 to nadal large-v3-turbo). Suma SHA-256 i rozmiar
`ggml-large-v3-turbo-q5_0.bin` do pobrania z API LFS Hugging Face przy implementacji (jak dla
pełnego w 3.2). Nowe pole `model_variant` w config.toml (`full` | `q5_0`, domyślnie `full`).
