# VoiceAsystent

Aplikacja paska menu macOS: nagrywasz głos skrótem albo kliknięciem ikony, a po
zakończeniu nagrania transkrypcja trafia do schowka — wklejasz ją `cmd+v` w dowolnej
aplikacji. Rozpoznawanie mowy działa w całości lokalnie (Whisper large-v3-turbo na GPU
Metal); nagranie ani tekst nie opuszczają komputera.

## Wymagania

- macOS 13 lub nowszy na Macu z Apple Silicon (M1 lub nowszy) — aplikacja używa GPU
  przez Metal, bez GPU nagrywanie jest zablokowane (praca na CPU nie jest wspierana).
- Ok. 2 GB wolnego miejsca na model rozpoznawania mowy (pobierany po instalacji).
- Dostęp do internetu tylko raz — do pobrania modelu. Potem aplikacja nie łączy się z siecią.

## Instalacja z obrazu .dmg

1. Otwórz `VoiceAsystent-<wersja>.dmg` i przeciągnij **VoiceAsystent.app** na skrót
   **Applications**.
2. Aplikacja jest podpisana lokalnie (bez notaryzacji Apple), więc przy pierwszym
   uruchomieniu pliku pobranego z sieci macOS pokaże ostrzeżenie Gatekeepera:
   - macOS 15 i nowsze: uruchom aplikację, zamknij ostrzeżenie, otwórz
     **Ustawienia systemowe → Prywatność i ochrona**, przewiń do sekcji Ochrona
     i kliknij **Otwórz mimo to** przy VoiceAsystent;
   - macOS 13–14: kliknij aplikację prawym przyciskiem w Finderze → **Otwórz** → **Otwórz**.
   Obraz zbudowany na tym samym komputerze (patrz „Budowanie ze źródeł") nie ma
   atrybutu kwarantanny i uruchamia się bez ostrzeżenia.
3. Po uruchomieniu w pasku menu (obok zegara) pojawia się **szare kółko**. Aplikacja nie
   ma okna ani ikony w Docku.

## Pierwsze uruchomienie

- **Pobieranie modelu.** Przy pierwszym starcie aplikacja pobiera model
  `ggml-large-v3-turbo.bin` (ok. 1,6 GB) z Hugging Face do
  `~/Library/Application Support/VoiceAsystent/models/`. Postęp widać w menu ikony
  (prawy przycisk): „Pobieranie modelu… 42%". Do zakończenia pobierania nagrywanie jest
  niedostępne — próba startu pokazuje komunikat. Przerwane pobieranie wznawia się przy
  następnym uruchomieniu albo po wybraniu **Ponów pobieranie** w menu; pobrany plik jest
  sprawdzany sumą SHA-256.
- **Zgoda na mikrofon.** Przy pierwszym nagraniu macOS pyta o dostęp do mikrofonu.
  Jeśli odmówisz, nagranie da ciszę i aplikacja pokaże powiadomienie z instrukcją:
  **Ustawienia systemowe → Prywatność i ochrona → Mikrofon → VoiceAsystent**.

## Używanie

| Działanie | Jak |
|-----------|-----|
| Start nagrywania | `ctrl+cmd+r` (w dowolnej aplikacji) albo kliknięcie szarego kółka |
| Stop i transkrypcja | `ctrl+cmd+s` albo kliknięcie czerwonego kółka |
| Wklejenie tekstu | `cmd+v` w miejscu z kursorem — schowek zawiera ostatnią transkrypcję |
| Wybór mikrofonu | prawy przycisk na ikonie → **Mikrofon** → urządzenie z listy |
| Zakończenie aplikacji | prawy przycisk na ikonie → **Zakończ** |

Podczas nagrywania kółko jest czerwone; w trakcie transkrypcji i w spoczynku — szare.
Pusta transkrypcja (cisza) nie zmienia zawartości schowka. Długie przerwy w mówieniu
(powyżej 1,5 s) są skracane przed transkrypcją, więc możesz spokojnie myśleć między
zdaniami. Nagranie ma limit długości, domyślnie 10 minut: po jego osiągnięciu kończy się
samo, dostajesz powiadomienie, a nagrany materiał jest transkrybowany do schowka tak samo
jak po `ctrl+cmd+s`.

Jeśli skrót jest już zajęty przez inną aplikację, menu ikony pokazuje o tym komunikat —
kliknięcie ikony działa wtedy nadal.

## Konfiguracja

Plik `~/Library/Application Support/VoiceAsystent/config.toml` powstaje przy pierwszej
zmianie mikrofonu; wszystkie pola są opcjonalne:

```toml
microphone = "MacBook Pro Microphone"   # brak = mikrofon domyślny systemu
language = "auto"                       # auto | pl | en
max_recording_secs = 600                # limit długości nagrania (auto-Stop + powiadomienie)
# model_path = "/inna/sciezka/ggml-large-v3-turbo.bin"

[silence]
threshold_rms = 0.01                    # próg ciszy przy przycinaniu nagrania
padding_ms = 200
```

Logi: `~/Library/Logs/VoiceAsystent/` (bez treści transkrypcji; nagrania nigdy nie są
zapisywane na dysk).

## Budowanie ze źródeł

Wymagane: Xcode Command Line Tools, Rust (stable, edycja 2024), `python3`.

```bash
scripts/build-app.sh   # → dist/VoiceAsystent.app (release, Metal, podpis ad-hoc)
scripts/build-dmg.sh   # → dist/VoiceAsystent-<wersja>.dmg (buduje też aplikację)
```

Szczegóły (testy, narzędzia deweloperskie `va-dev`, rozwiązywanie problemów):
[docs/RUNBOOK.md](docs/RUNBOOK.md).
