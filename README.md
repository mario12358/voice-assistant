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
| Wcześniejsza wypowiedź | prawy przycisk na ikonie → **Historia** → pozycja z listy (kopiuje jej pełny tekst do schowka, potem `cmd+v`) |
| Wyczyszczenie historii | prawy przycisk na ikonie → **Historia** → **Wyczyść historię** |
| Wybór mikrofonu | prawy przycisk na ikonie → **Mikrofon** → urządzenie z listy |
| Język i limit nagrania | prawy przycisk → **Ustawienia** → **Język** (automatycznie / polski / angielski) albo **Limit nagrania** (5–30 min); działa od następnego nagrania, bez restartu |
| Uruchamianie przy logowaniu | prawy przycisk → **Ustawienia** → **Uruchamiaj przy logowaniu** (tylko w aplikacji z folderu Aplikacje) |
| Logi | prawy przycisk → **Pokaż logi** (Finder; pliki starsze niż 7 dni są usuwane przy starcie) |
| Zakończenie aplikacji | prawy przycisk na ikonie → **Zakończ** |

**Historia wypowiedzi.** Każda niepusta transkrypcja trafia na listę w podmenu **Historia**
(godzina i początek tekstu, najnowsza na górze, 30 ostatnich). Lista jest zapisywana w pliku
`~/Library/Application Support/VoiceAsystent/history.json`, dostępnym tylko dla Twojego
użytkownika, i wraca po ponownym uruchomieniu aplikacji. To jedyne miejsce, w którym tekst
Twoich wypowiedzi ląduje na dysku; **Wyczyść historię** kasuje listę razem z plikiem.

**Model.** Podmenu **Model** pokazuje, jaki model jest używany, jego rozmiar i stan
(`large-v3-turbo · 1,6 GB · gotowy`, w trakcie pobierania procent). **Pokaż w Finderze**
zaznacza plik modelu. **Usuń model…** po potwierdzeniu w tym samym menu kasuje plik (zwalnia
ok. 1,6 GB); nagrywanie jest wtedy niedostępne do czasu, aż wybierzesz **Pobierz ponownie**,
które pobiera model od nowa z postępem i uruchamia nagrywanie bez restartu aplikacji. Usuwanie
jest nieaktywne w trakcie pobierania, nagrywania i transkrypcji. Jeśli w `config.toml` ustawisz
własną ścieżkę `model_path`, podmenu pokazuje ten plik, a usuwanie i pobieranie są wyłączone.

W tym samym podmenu wybierasz wariant modelu: **Pełny (1,6 GB)** albo **Skwantyzowany q5_0
(0,6 GB)** — ten drugi to ten sam large-v3-turbo w lżejszej wersji (szybsze pobranie, mniej
pamięci GPU, minimalnie niższa jakość). Wybór niepobranego wariantu pobiera go z postępem,
a pobranego tylko ładuje (chwilę widać wtedy „Pobieranie modelu… 0%”); nagrywanie wraca bez
restartu. Gdy oba warianty leżą na dysku, **Usuń nieużywany wariant** zwalnia miejsce po tym,
którego nie używasz.

**Szybki start.** Suma kontrolna modelu jest liczona tylko po zmianie pliku (znacznik
`*.verified` obok modelu), więc ikona pojawia się od razu, a model ładuje się na GPU w tle.

**Sygnał „gotowe”.** Po każdej transkrypcji zapisanej do schowka pojawia się powiadomienie
„Transkrypcja w schowku” z początkiem tekstu (wyłączysz je w `config.toml`, tam też włączysz
dźwięk). Szum bez mowy daje pusty wynik zamiast wymyślonych słów.

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
notify_on_transcript = true             # powiadomienie „Transkrypcja w schowku” z początkiem tekstu
sound_on_transcript = false             # dźwięk systemowy po zapisie do schowka
model_variant = "full"                  # full (1,6 GB) | q5_0 (0,6 GB) — też z menu Model
# model_path = "/inna/sciezka/ggml-large-v3-turbo.bin"   # własny plik GGML: bez pobierania i sumy SHA-256

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
