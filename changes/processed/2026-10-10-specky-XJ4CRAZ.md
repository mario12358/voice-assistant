---
data: 2026-10-10
rodzaj: błąd
priorytet: normalny
dotyczy: ogólne (nowe wymaganie Specky, sekcja „Rozpoznawanie mowy”; kod: apps/voice-asystent/src/startup.rs, README sekcja Konfiguracja)
specky_req: 01M4K6M32Y0MQ3JPWM4XJ4CRAZ
specky_hash: 9f73318
---

# VA-MODEL-3 — aplikacja honoruje `model_path` z config.toml

Nowe wymaganie zaakceptowane przez właściciela 2026-10-10 (wsad 01M4K6JNNT5ZPGGVCXY8D7AMAP),
powód z kolejki: `to_build` (status `open`). Rodzaj: **błąd** — pole istnieje od zadania 1.3
i jest opisane w README, ale aplikacja paska menu (startup z zadania 5.1) sprawdza i ładuje
model wyłącznie ze stałego katalogu; honoruje je tylko `va-dev transcribe`. Wykryte przez: ralph
(przegląd obsługi modelu 2026-10-10).

**Treść (v1 @9f73318):** Jeśli config.toml zawiera model_path, aplikacja paska menu używa tego
pliku zamiast modelu z katalogu domyślnego i nie pobiera niczego; dziś to pole honoruje tylko
narzędzie va-dev, a README opisuje je jako działające.

**Kryteria:**
1. (crit: 01M4K6M33C1BCHBQ7DEG4GER2W) Z model_path wskazującym istniejący plik GGML aplikacja ładuje ten plik, nie pobiera modelu i w podmenu „Model” pokazuje jego nazwę pliku i ścieżkę
2. (crit: 01M4K6M33CWKR54XAT4JVG3HWG) Z model_path wskazującym nieistniejący plik aplikacja pokazuje w menu „Brak modelu” z tą ścieżką i nie pobiera modelu domyślnego
3. (crit: 01M4K6M33CMHX4SQDRACYG60ZV) Przy własnej ścieżce „Usuń model” i „Pobierz ponownie” są nieaktywne (plik należy do użytkownika)
4. (crit: 01M4K6M33CGTMKF9VPRYQXKE1C) Bez model_path zachowanie jak dotąd (VA-MODEL-1)

**„Było vs jest”:** `startup::check_model(paths)` i `ModelDownload::needed` używają tylko
`ModelStore::new(paths.models_dir, LARGE_V3_TURBO)`. Ma być: źródło modelu wyliczane z konfiguracji
(domyślny magazyn albo własna ścieżka), własna ścieżka bez sumy SHA-256 i bez pobierania.
