# ralph/PLAN.md - Lista zadań projektu

## Informacje o projekcie

- **Nazwa**: VoiceAsystent — aplikacja macOS w pasku menu: nagranie głosu skrótem lub kliknięciem → transkrypcja Whisper → tekst w schowku (wklejanie cmd+v)
- **Platforma**: wyłącznie macOS (Apple Silicon), dystrybucja jako .dmg z VoiceAsystent.app — Windows/Linux poza zakresem (decyzja właściciela 2026-10-08)
- **Stack**: Rust (workspace Cargo), whisper-rs (whisper.cpp) + Whisper large-v3-turbo (GGML) tylko na GPU Metal, cpal + rubato (audio), tray-icon + muda + tao (pasek menu, pętla zdarzeń), global-hotkey (skróty globalne), arboard (schowek), serde/toml (konfiguracja), thiserror/anyhow, tracing
- **Źródło wymagań**: Specky, projekt 01M4EJNFTHDZ3ECMHT7425APR0 (jedyne źródło prawdy). Identyfikatory `[VA-…]` w zadaniach = `external_ref` wymagań; pod zadaniem kotwice `- Specky: (req: …)`
- **Uwaga o tagach tech**: brak `rust.md` w ralph-guidelines/ — zadania Rust ładują tylko `general.md`; konwencje Rust ustala zadanie 1.1 w PROJECT_CONTEXT.md

## Zadania

<!-- Linia zadania ≤ 400 znaków (walidator ralph-start.sh ostrzega powyżej). Plan mówi CO     -->
<!-- jest do zrobienia; uzasadnienia, ustalenia z realizacji i „dlaczego tak" idą do          -->
<!-- ralph/REPORT.md (Log problemów) albo ralph/PROJECT_CONTEXT.md (Kluczowe decyzje).        -->
<!-- Powód: plan jest czytany i edytowany przy KAŻDYM zadaniu i w całości trafia do prompta,  -->
<!-- a tamte pliki rotują się do archiwum. Kryteria akceptacji zapisuj jako `- AC:` pod       -->
<!-- zadaniem, nie doklejaj ich do jego linii.                                                -->
<!--                                                                                          -->
<!-- Każda pozycja `- [ ]` MUSI zaczynać się od słowa `Zadanie` albo `Test` (znacznik stanu     -->
<!-- przed nim jest OK: `- [ ] ⛔ Zadanie 4.2: …`). Pozycja bez tego słowa jest dla pętli        -->
<!-- niewidzialna — nie da się jej wybrać i nie liczy się do postępu fazy, a wygląda jak         -->
<!-- zrobiona robota; walidator ostrzega. Notatkę zapisz w ralph/REPORT.md.                      -->
<!-- `⛔` znaczy „czeka na człowieka" (test manualny na produkcji, decyzja właściciela):          -->
<!-- zadanie liczy się do kompletności fazy, ale podpowiedź startowa je pomija, żeby pętla       -->
<!-- nie wskazywała co sesję tego samego, czego sama nie zrobi.                                  -->
<!--                                                                                          -->
<!-- Nagłówek `### Faza N: … — ✅ ukończona` z komentarzem `ralph-archived` pod spodem to ślad   -->
<!-- fazy przeniesionej do ralph/PLAN_ARCHIVE.md przez rotację plików stanu. Nie edytuj go     -->
<!-- ręcznie — zależności (wymaga: X.Y) do wymienionych tam zadań nadal się rozwiązują.        -->

### Faza 1: Setup i fundamenty

- [x] Zadanie 1.1: Szkielet architektury i konwencje — mapa crate'ów (config, audio, model, stt, clipboard, app), typy błędów (thiserror w lib, anyhow w bin), traity dla wymiennych elementów (źródło audio, STT, schowek), konwencje nazw i testów; wynik do PROJECT_CONTEXT.md [VA-PLAT-1, VA-TECH-1]
  - Specky: (req: 01M4EKHBS1EMNP53JTYKVA3YTB v1 @41e9b47)
  - Specky: (req: 01M4EKHC5VP9R6P1EY8E55S2XE v1 @940ad46)
  - AC: PROJECT_CONTEXT.md zawiera mapę crate → odpowiedzialność, zasady błędów/logowania, ścieżki macOS (Application Support, Logs) i sposób dostarczenia modelu (pobierany po instalacji)
- [x] Zadanie 1.2: Workspace Cargo z crate'ami z 1.1, binarka `voice-asystent` (aplikacja) i `va-dev` (narzędzia deweloperskie), target tylko macOS (`compile_error!` poza macOS), `rustfmt.toml`, clippy `-D warnings`, `.gitignore`; uzupełnij pola Testy/Linter w ralph/config.md (wymaga: 1.1) [VA-PLAT-1, VA-TECH-1]
  - Specky: (req: 01M4EKHBS1EMNP53JTYKVA3YTB v1 @41e9b47)
  - Specky: (req: 01M4EKHC5VP9R6P1EY8E55S2XE v1 @940ad46)
  - AC: `cargo build --workspace`, `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo fmt --check` przechodzą
- [x] Test: 1.2 — smoke test kompilacji workspace + `va-dev --version`
- [x] Zadanie 1.3: Konfiguracja TOML w `~/Library/Application Support/VoiceAsystent/config.toml`: wybrany mikrofon (nazwa), ścieżka modelu, język transkrypcji (domyślnie auto), progi ciszy; wartości domyślne; zapis zmian z aplikacji (wymaga: 1.2) [VA-REC-4]
  - Specky: (req: 01M4EKHCZDHBRC6DY70CRB55G3 v1 @956c038)
  - AC: brak pliku → wartości domyślne; niepoprawny TOML → log błędu z numerem linii i start na domyślnych
  - AC: zapis wyboru mikrofonu i ponowny odczyt zwraca tę samą wartość
  - AC: wejście — start aplikacji (odczyt) i wybór mikrofonu w menu z 5.3 (zapis)
- [x] Test: 1.3 — parsowanie pełnego/pustego/błędnego pliku, zapis i odczyt w katalogu tymczasowym
- [x] Zadanie 1.4: Logowanie `tracing` do `~/Library/Logs/VoiceAsystent/` (rotacja dzienna) i stderr; spany na etapach nagranie/transkrypcja z czasem; treść transkrypcji tylko na poziomie debug; audio nie trafia na dysk (wymaga: 1.2) (tech: observability) [VA-STT-1]
  - Specky: (req: 01M4EKHCCRE1GSXSNNWR8CMD0M v1 @fd1ba16)
  - AC: domyślny poziom info nie loguje treści transkrypcji
- [x] Test: 1.4 — przechwycenie logów: treść transkrypcji nieobecna na poziomie info

### Faza 2: Audio — mikrofony, nagrywanie, cisza

- [x] Zadanie 2.1: Crate audio: lista mikrofonów (cpal, host CoreAudio) z oznaczeniem domyślnego; wybór po nazwie z konfiguracji, a gdy go brak — mikrofon domyślny systemu z ostrzeżeniem w logu (wymaga: 1.3) [VA-REC-4]
  - Specky: (req: 01M4EKHCZDHBRC6DY70CRB55G3 v1 @956c038)
  - AC: zniknięty mikrofon z konfiguracji → użyty domyślny systemowy
  - AC: brak jakiegokolwiek mikrofonu → czytelny błąd
- [x] Test: 2.1 — wybór urządzenia po nazwie i fallback na domyślne (abstrakcja hosta audio, bez sprzętu)
- [x] Zadanie 2.2: Nagrywanie do bufora w pamięci (start/stop) z dowolnego formatu urządzenia → 16 kHz mono f32 (downmix + resampling rubato); limit długości nagrania z konfiguracji (domyślnie 5 min) (wymaga: 2.1) [VA-STT-1]
  - Specky: (req: 01M4EKHCCRE1GSXSNNWR8CMD0M v1 @fd1ba16)
  - AC: wynik zawsze 16 kHz mono f32 niezależnie od formatu wejścia (44.1/48 kHz, stereo)
  - AC: nagranie nie trafia na dysk
- [x] Test: 2.2 — konwersja syntetycznego sygnału 48 kHz stereo → 16 kHz mono (długość, częstotliwość tonu zachowana)
- [x] Zadanie 2.3: Przycinanie ciszy na początku/końcu nagrania (energetyczny VAD, progi z konfiguracji); nagranie samej ciszy → wynik pusty (wymaga: 2.2) [VA-REC-3] (pr: #1)
  - Specky: (req: 01M4EKHCXK9BETN4Q065X24ES9 v1 @523b4a6)
  - AC: nagranie samej ciszy → brak wywołania STT, schowek bez zmian
- [x] Test: 2.3 — fixtures WAV (cisza, mowa z ciszą, sama cisza) → oczekiwane granice segmentu (pr: #1)
- [x] Zadanie 2.4: Komenda `va-dev mic-test [--seconds N] [--save plik.wav]`: lista mikrofonów, nagranie N s, poziom sygnału w terminalu; zapis WAV tylko przy jawnym `--save` (wymaga: 2.2, 2.3) (pr: #2)
  - AC: wejście — komenda `va-dev mic-test`
- [x] Test: 2.4 — zapis WAV z bufora (hound) i odczyt z powrotem (pr: #2)

### Faza 3: Transkrypcja — Whisper large-v3-turbo na Metal

- [x] Zadanie 3.1: Weryfikacja GPU Metal przy starcie (obecność urządzenia Metal, build z feature `metal`); brak → komunikat „Wymagane GPU (Metal) — praca na CPU nie jest wspierana", transkrypcja zablokowana (wymaga: 1.2) (spec: WYTYCZNE_TECHNICZNE.md) [VA-STT-2] (pr: #5)
  - Specky: (req: 01M4EKHCM2WDGKPH824D8532JD v1 @5e21e03)
  - AC: brak Metal → komunikat i brak wywołania STT na CPU
  - AC: wejście — start aplikacji i `va-dev transcribe`
- [x] Test: 3.1 — logika decyzji (brak GPU / OK) na wstrzykniętych danych o urządzeniu (pr: #5)
- [x] Zadanie 3.2: Crate model: pobieranie GGML large-v3-turbo z Hugging Face do `~/Library/Application Support/VoiceAsystent/models/` (wznawianie przez HTTP Range, postęp przez callback, weryfikacja SHA-256, plik `.part` → atomowa zmiana nazwy); sprawdzenie obecności i sumy przy starcie; `va-dev model-download` (wymaga: 1.3) (spec: WYTYCZNE_TECHNICZNE.md) [VA-STT-1, VA-MODEL-1] (pr: #8)
  - Specky: (req: 01M4EKHCCRE1GSXSNNWR8CMD0M v1 @fd1ba16)
  - Specky: (req: 01M4EQH9D30871ZG5JMWK8VHFV v1 @5a6280e)
  - AC: zła suma SHA-256 → plik usunięty, błąd; ponowna próba możliwa
  - AC: przerwane pobieranie wznawiane od miejsca przerwania; istniejący poprawny model → brak pobierania
  - AC: wejście — `va-dev model-download` i start aplikacji bez modelu (5.6)
- [x] Test: 3.2 — lokalny serwer HTTP w teście: pobranie, wznowienie po przerwaniu (Range), zła suma, „już pobrany” (pr: #8)
- [x] Zadanie 3.3: Crate stt: trait `SpeechToText` + implementacja whisper-rs z backendem Metal, model ładowany raz przy starcie, język z konfiguracji (auto | pl | en), zwraca tekst + czas inferencji; mock do testów (wymaga: 3.1, 3.2) (spec: WYTYCZNE_TECHNICZNE.md) [VA-STT-1, VA-STT-2] (pr: #9)
  - Specky: (req: 01M4EKHCCRE1GSXSNNWR8CMD0M v1 @fd1ba16)
  - Specky: (req: 01M4EKHCM2WDGKPH824D8532JD v1 @5e21e03)
  - AC: model ładowany jednokrotnie; kolejne transkrypcje bez ponownego ładowania
  - AC: inferencja na Metal (log backendu przy starcie)
- [x] Test: 3.3 — test `#[ignore]` (model + GPU): fixtures WAV PL i EN → transkrypcja zawiera oczekiwane słowa kluczowe (pr: #9)
- [x] Zadanie 3.4: Komenda `va-dev transcribe <plik.wav>` (WAV → normalizacja 2.2 → cisza 2.3 → STT → stdout, z czasem inferencji) (wymaga: 2.3, 3.3) (pr: #10)
  - AC: wejście — komenda `va-dev transcribe plik.wav`
- [x] Test: 3.4 — integracja z mockiem STT: WAV 44.1 kHz stereo trafia do STT jako 16 kHz mono (pr: #10)

### Faza 4: Rdzeń — stan nagrywania i schowek

- [x] Zadanie 4.1: Crate clipboard: trait `TextSink` + implementacja schowka systemowego (arboard); zapis zastępuje poprzednią zawartość; pusty tekst → schowek bez zmian (wymaga: 1.2) [VA-REC-3] (pr: #13)
  - Specky: (req: 01M4EKHCXK9BETN4Q065X24ES9 v1 @523b4a6)
  - AC: schowek zawiera dokładnie tekst transkrypcji (z polskimi znakami)
  - AC: pusta transkrypcja nie zmienia schowka
- [x] Test: 4.1 — logika z mockiem schowka; test prawdziwego schowka `#[ignore]` (sesja graficzna) (pr: #13)
- [x] Zadanie 4.2: Automat stanów aplikacji Idle → Recording → Transcribing → Idle (oraz Error → Idle) z poleceniami Start/Stop niezależnymi od źródła (skrót, kliknięcie); Start w Recording/Transcribing i Stop w Idle są ignorowane z logiem (wymaga: 1.2) [VA-REC-1, VA-REC-2, VA-UI-2] (pr: #15)
  - Specky: (req: 01M4EKHCQYDSN2280XE8DMXR08 v1 @5c4517a)
  - Specky: (req: 01M4EKHCVH9GSS063H0MNZX2F9 v1 @1d726bc)
  - Specky: (req: 01M4EKHD3Y6M9V9E01V27AH18Z v1 @a4fdb35)
  - AC: drugi Start w trakcie nagrywania nie tworzy drugiego nagrania; Stop bez nagrania nic nie robi
- [x] Test: 4.2 — tablica przejść stanów na wstrzykniętych poleceniach (pr: #15)
- [x] Zadanie 4.3: Kontroler: Start → nagrywanie (2.2); Stop → przycięcie ciszy (2.3) → STT (3.3) w wątku roboczym → TextSink (4.1); zmiany stanu publikowane kanałem do UI; błąd jednej transkrypcji → Error → Idle bez zamykania aplikacji (wymaga: 2.3, 3.3, 4.1, 4.2) [VA-REC-2, VA-REC-3] (pr: #16)
  - Specky: (req: 01M4EKHCVH9GSS063H0MNZX2F9 v1 @1d726bc)
  - Specky: (req: 01M4EKHCXK9BETN4Q065X24ES9 v1 @523b4a6)
  - AC: wejście — polecenia Start/Stop z 5.2 i 5.4
  - AC: po Stop tekst transkrypcji trafia do schowka; UI dostaje zdarzenia zmiany stanu
- [x] Test: 4.3 — integracja z mockami (audio z fixture WAV, STT, schowek): tekst w schowku, błąd STT → Error → Idle (pr: #16)

### Faza 5: Interfejs macOS — pasek menu i skróty globalne

- [x] Zadanie 5.1: Binarka `voice-asystent` jako aplikacja paska menu (tao event loop, bez ikony w Docku — `ActivationPolicy::Accessory`): ikona szare kółko w Idle, czerwone w Recording (Transcribing — szare z podpowiedzią „Transkrypcja…"), ikony jako zasoby PNG @1x/@2x (wymaga: 4.3) [VA-UI-1] (pr: #18)
  - Specky: (req: 01M4EKHD1BQYRNWBAVRA5FJTRC v1 @df8032b)
  - AC: po starcie szare kółko; zdarzenie Recording → czerwone; powrót do Idle → szare
  - AC: wejście — zdarzenia zmiany stanu z kontrolera 4.3
- [x] Test: 5.1 — mapowanie stanów kontrolera na ikonę i podpowiedź (bez GUI) (pr: #18)
- [x] Zadanie 5.2: Lewe kliknięcie ikony: szare → Start, czerwone → Stop (menu tylko pod prawym kliknięciem — `menu_on_left_click(false)`) (wymaga: 5.1) [VA-UI-2] (pr: #19)
  - Specky: (req: 01M4EKHD3Y6M9V9E01V27AH18Z v1 @a4fdb35)
  - AC: wejście — kliknięcie ikony w pasku menu
  - AC: kliknięcie czerwonego kółka kończy nagranie i transkrypcja trafia do schowka
- [x] Test: 5.2 — obsługa zdarzeń kliknięcia → polecenia kontrolera (zdarzenia wstrzyknięte) (pr: #19)
- [x] Zadanie 5.3: Menu pod prawym kliknięciem: podmenu „Mikrofon" z listą dostępnych urządzeń (zaznaczony wybrany, odświeżane przy otwarciu), „Zakończ"; wybór zapisuje konfigurację (1.3) i działa od następnego nagrania (wymaga: 2.1, 5.1) [VA-REC-4] (pr: #20)
  - Specky: (req: 01M4EKHCZDHBRC6DY70CRB55G3 v1 @956c038)
  - AC: wejście — prawe kliknięcie ikony → Mikrofon → nazwa urządzenia
  - AC: wybór przetrwa restart aplikacji
- [x] Test: 5.3 — budowanie modelu menu z listy urządzeń i obsługa wyboru (bez GUI) (pr: #20)
- [x] Zadanie 5.4: Skróty globalne (global-hotkey): ctrl+cmd+r → Start, ctrl+cmd+s → Stop, działające przy dowolnej aktywnej aplikacji; nieudana rejestracja (konflikt) → komunikat w menu i logu (wymaga: 4.3, 5.1) [VA-REC-1, VA-REC-2] (pr: #21)
  - Specky: (req: 01M4EKHCQYDSN2280XE8DMXR08 v1 @5c4517a)
  - Specky: (req: 01M4EKHCVH9GSS063H0MNZX2F9 v1 @1d726bc)
  - AC: wejście — naciśnięcie ctrl+cmd+r / ctrl+cmd+s w dowolnej aplikacji
- [x] Test: 5.4 — mapowanie zdarzeń skrótów na polecenia kontrolera (zdarzenia wstrzyknięte) (pr: #21)
- [x] Zadanie 5.5: Uprawnienie mikrofonu: odmowa dostępu → powiadomienie z instrukcją (Ustawienia systemowe → Prywatność → Mikrofon), stan Error → Idle; brak modelu / brak Metal → stała pozycja z komunikatem w menu (wymaga: 3.1, 3.2, 5.3) [VA-PLAT-2, VA-STT-2] (pr: #22)
  - Specky: (req: 01M4EKHC2HZ06DJWGX8EB7QCWD v1 @81be39b)
  - Specky: (req: 01M4EKHCM2WDGKPH824D8532JD v1 @5e21e03)
  - AC: wejście — próba Start bez uprawnienia; start aplikacji bez modelu/GPU
- [x] Test: 5.5 — mapowanie błędów (brak uprawnienia, brak modelu, brak GPU) na komunikaty (bez GUI) (pr: #22)
- [x] Zadanie 5.6: Pobieranie modelu po instalacji: przy starcie bez modelu aplikacja w tle pobiera go (3.2), postęp w menu („Pobieranie modelu… 42%”) i podpowiedzi ikony; Start w tym czasie → komunikat zamiast nagrania; błąd → pozycja menu „Ponów pobieranie”; po pobraniu ładowanie STT (wymaga: 3.2, 4.3, 5.5) [VA-MODEL-1] (pr: #23)
  - Specky: (req: 01M4EQH9D30871ZG5JMWK8VHFV v1 @5a6280e)
  - AC: wejście — pierwsze uruchomienie aplikacji bez modelu
  - AC: po pobraniu aplikacja nie wykonuje ruchu sieciowego
- [x] Test: 5.6 — automat stanów pobierania (brak → pobieranie → gotowy / błąd → ponów) z mockiem pobierania; Start podczas pobierania odrzucony (pr: #23)

### Faza 6: Instalator .dmg i weryfikacja

- [x] Zadanie 6.1: Bundle VoiceAsystent.app: Info.plist (CFBundleIdentifier, LSUIElement=true, NSMicrophoneUsageDescription po polsku, LSMinimumSystemVersion), ikona aplikacji, bez modelu (pobierany po instalacji — 5.6), podpis ad-hoc (`codesign -s -`); skrypt `scripts/build-app.sh` (release, `--features metal`) (wymaga: 5.6) [VA-PLAT-2, VA-MODEL-1] (pr: #26)
  - Specky: (req: 01M4EKHC2HZ06DJWGX8EB7QCWD v1 @81be39b)
  - Specky: (req: 01M4EQH9D30871ZG5JMWK8VHFV v1 @5a6280e)
  - AC: `scripts/build-app.sh` tworzy `dist/VoiceAsystent.app`, która uruchamia się z Findera i pokazuje ikonę w pasku menu
- [x] Test: 6.1 — walidacja wygenerowanego Info.plist (`plutil -lint`) i struktury bundla w teście skryptu (pr: #26)
- [x] Zadanie 6.2: Skrypt `scripts/build-dmg.sh`: obraz `dist/VoiceAsystent-<wersja>.dmg` (hdiutil) z aplikacją i skrótem do /Applications; wersja z `git describe` (wymaga: 6.1) [VA-PLAT-2] (pr: #27)
  - Specky: (req: 01M4EKHC2HZ06DJWGX8EB7QCWD v1 @81be39b)
  - AC: zamontowany .dmg zawiera VoiceAsystent.app i link do Applications, bez pliku modelu
- [x] Test: 6.2 — test skryptu: dmg powstaje, montuje się (`hdiutil attach -nobrowse`) i zawiera oczekiwane pliki (pr: #27)
- [x] Zadanie 6.3: README + docs/RUNBOOK.md: wymagania (macOS na Apple Silicon), budowanie, instalacja z .dmg, pierwsze uruchomienie z pobraniem modelu (~1,6 GB, Gatekeeper dla aplikacji bez notaryzacji, zgoda na mikrofon), skróty (wymaga: 6.2) (pr: #28)
- [x] Zadanie 6.4: Test manualny właściciela: instalacja z .dmg, nagranie skrótami i kliknięciem ikony, wklejenie cmd+v w kilku aplikacjach, zmiana mikrofonu — wynik do REPORT.md (wymaga: 6.2)

### Faza 7: Historia wypowiedzi, pauzy w nagraniu, limit długości

- [x] Zadanie 7.1: `va-audio`: `compress_pauses(samples, rate, &SilenceParams)` — odcinki ciszy wewnątrz nagrania dłuższe niż 1,5 s skracane do 0,5 s (stałe w module), mowa i krótsze pauzy bajt w bajt bez zmian; `trim_silence` nietknięte [VA-REC-5] (zmiana: 2026-10-10-specky-1BY4F4Z.md) (pr: #33)
  - Specky: (req: 01M4K06AQB7B8055HQ21BY4F4Z v1 @04c88f7)
  - Specky kryteria: (crit: 01M4K06AR8A386FPF759VJJ9M2) pauza > 1,5 s → 0,5 s; (crit: 01M4K06AR8A4107QK83WR0K6YC) reszta identyczna; (crit: 01M4K06AR8VG4GQ4HY90VTX40G) brzegi jak dotąd
  - AC: sygnał mowa–40 s ciszy–mowa po przetworzeniu ma 0,5 s ciszy między fragmentami, a fragmenty mowy są identyczne z wejściem; pauza 1,0 s zostaje; sama cisza nadal daje pusty wycinek
- [x] Test: 7.1 — testy jednostkowe na sygnale syntetycznym (pauza 40 s, 1,0 s, kilka pauz, brzegi), mutacja progu 1,5 s czerwona (pr: #33)
- [x] Zadanie 7.2: Wpięcie `compress_pauses` po `trim_silence` w kontrolerze (`stop_and_transcribe`) i w `va-dev transcribe`; fixture WAV „dwa zdania rozdzielone 40 s ciszy” (skrypt `tests/fixtures/generate_pause_fixture.py`) (wymaga: 7.1) [VA-REC-5] (zmiana: 2026-10-10-specky-1BY4F4Z.md) (pr: #34)
  - Specky: (req: 01M4K06AQB7B8055HQ21BY4F4Z v1 @04c88f7)
  - Specky kryteria: (crit: 01M4K06AR8TJXY24K05X0MNBJC) 2 zdania + 40 s ciszy → oba zdania bez dodatkowych fraz
  - AC: wejście — Stop w kontrolerze (ścieżka Recorder → STT) oraz komenda `va-dev transcribe`; mutacja (pominięcie compress_pauses) wykrywana testem kontrolera przez długość próbek przekazanych do STT
- [x] Test: 7.2 — test kontrolera z fałszywą nagrywarką (STT dostaje skrócone pauzy) + `#[ignore]` prawdziwa transkrypcja fixture z 40 s ciszy (oba zdania, nic między nimi) (pr: #34)
- [x] Zadanie 7.3: Limit nagrania: domyślne `max_recording_secs` 600 (va-config); `Recorder` zgłasza osiągnięcie limitu (callback/kanał z wątku audio), kontroler dostaje `Message::LimitReached` → `Input::Stop` tą samą ścieżką co ctrl+cmd+s + `ControllerEvent::LimitReached` [VA-REC-6] (zmiana: 2026-10-10-specky-9VFYND0.md) (pr: #35)
  - Specky: (req: 01M4K06AXGNSZ9XEZ3E9VFYND0 v1 @97b27f2)
  - Specky kryteria: (crit: 01M4K06AY61DWRJ5R2DFNDJD6W) domyślnie 600 s, config nadal zmienia; (crit: 01M4K06AY60EVTQQ78X8RQ4RCK) auto-Stop → transkrypcja → schowek, ikona szara; (crit: 01M4K06AY6N6SB7WBZE4J4MM09) ręczny Stop jak dotąd
  - AC: wejście — zdarzenie limitu z nagrywarki (bez polecenia użytkownika) kończy nagranie: automat Recording → Transcribing → Idle, STT dostaje cały bufor do limitu, schowek otrzymuje tekst; ręczny Stop przed limitem bez zmian
- [x] Test: 7.3 — `Config::default().max_recording_secs == 600` i odczyt z pliku; test kontrolera: fałszywa nagrywarka emituje limit → Stop bez polecenia, STT i schowek wywołane, stan Idle; mutacja (ignorowanie limitu) czerwona (pr: #35)
- [x] Zadanie 7.4: Aplikacja: `ControllerEvent::LimitReached` → powiadomienie systemowe „Osiągnięto limit długości nagrania” (messages.rs, tekst po polsku, nazwa limitu w minutach); README/RUNBOOK: limit 10 min i auto-Stop (wymaga: 7.3) [VA-REC-6] (zmiana: 2026-10-10-specky-9VFYND0.md) (pr: #36)
  - Specky: (req: 01M4K06AXGNSZ9XEZ3E9VFYND0 v1 @97b27f2)
  - Specky kryteria: (crit: 01M4K06AY6B3424GJDBAQQK3WY) powiadomienie przy auto-Stop
  - AC: wejście — zdarzenie kontrolera w pętli tao (`UserEvent::Controller`) wywołuje `notify`; tekst powiadomienia zawiera limit w minutach z konfiguracji
- [x] Test: 7.4 — test messages (tytuł/treść z limitem) i test wpięcia (mapowanie zdarzenia → Problem) w module app (pr: #36)
- [x] Zadanie 7.5: `va-core::history`: bufor 30 wpisów `{text, at}` (najnowszy pierwszy), zapis JSON do `Paths::history_file` (tryb 0600, atomowo), `clear()` kasuje plik; kontroler: po `Delivery::Written` wpis + `ControllerEvent::HistoryChanged`, polecenia `CopyHistoryEntry(id)` (przez `TextSink`) i `ClearHistory`; treść poza logami [VA-HIST-1] (zmiana: 2026-10-10-specky-CY0SQA5.md) (pr: #37)
  - Specky: (req: 01M4K06AGAV8G79RJMXCY0SQA5 v1 @d3eecdb)
  - Specky kryteria: (crit: 01M4K06AH032985T6JXJX7EE3W) wpis po niepustej, cisza bez wpisu; (crit: 01M4K06AH0FMZJZZX7SHA9KR5A) plik 0600, wraca po restarcie; (crit: 01M4K06AH0BR7ZQYXPH6B08P26) wyczyść kasuje z dysku; (crit: 01M4K06AH0B2HW26D29WR4H4F3) treść poza logami
  - AC: wejście — zakończona transkrypcja w kontrolerze (ta sama ścieżka co schowek) tworzy wpis; `CopyHistoryEntry` dostarcza pełny tekst do `TextSink` bez przejścia automatu do Recording; nowy kontroler z tym samym plikiem publikuje te same wpisy; `ClearHistory` zostawia pustą listę i brak pliku
- [x] Test: 7.5 — testy jednostkowe historii (limit 30, kolejność, round-trip pliku, uprawnienia 0600, clear) + testy kontrolera (wpis po Written, brak po SkippedEmpty, kopiowanie, czyszczenie); mutacja limitu 30 czerwona; test, że log nie zawiera tekstu wpisu (pr: #37)
- [x] Zadanie 7.6: Menu ikony: podmenu „Historia” (`HH:MM · początek tekstu…` do 40 znaków, najnowsza na górze, „Brak wpisów” gdy pusto, „Wyczyść historię”), odświeżane na `HistoryChanged`; zdarzenia menu → `CopyHistoryEntry` / `ClearHistory`; README o historii (wymaga: 7.5) [VA-HIST-1] (zmiana: 2026-10-10-specky-CY0SQA5.md) (pr: #38)
  - Specky: (req: 01M4K06AGAV8G79RJMXCY0SQA5 v1 @d3eecdb)
  - Specky kryteria: (crit: 01M4K06AH0CZS3Z7N0FXXSKE1T) podmenu z godziną i początkiem tekstu, od najnowszego; (crit: 01M4K06AH09HZHH764X48P4GM4) kliknięcie kopiuje pełny tekst bez nagrywania
  - AC: wejście — kliknięcie pozycji podmenu w pętli tao (`UserEvent::Menu`) wysyła `CopyHistoryEntry` z właściwym id; funkcja budująca etykiety i parsująca id menu testowana jednostkowo (jak `microphone_items`)
- [x] Test: 7.6 — testy etykiet (skrót 40 znaków, godzina, kolejność, pusta lista) i mapowania id menu → polecenie; mutacja kolejności czerwona (pr: #38)
- [ ] ⛔ Zadanie 7.7: Test manualny właściciela: historia (wpisy po nagraniach, kopiowanie starszego wpisu, restart, „Wyczyść historię”), nagranie z pauzami po 30–40 s bez śmieci w tekście, limit (tymczasowo `max_recording_secs = 20`: auto-Stop, powiadomienie, tekst w schowku) — wynik do REPORT.md (wymaga: 7.2, 7.4, 7.6)

## Pokrycie spec

Źródłem wymagań jest Specky (projekt 01M4EJNFTHDZ3ECMHT7425APR0). `spec/` zawiera tylko WYTYCZNE_TECHNICZNE.md (Rust, large-v3-turbo, tylko GPU) i niewypełnione szablony.

| Wymaganie (external_ref) | Treść w skrócie | Zadania |
|--------------------------|-----------------|---------|
| VA-PLAT-1 | tylko macOS | 1.1, 1.2 |
| VA-PLAT-2 | instalacja z .dmg, zgoda na mikrofon | 5.5, 6.1, 6.2, 6.3, 6.4 |
| VA-TECH-1 | Rust | 1.1, 1.2 |
| VA-STT-1 | lokalnie, large-v3-turbo, nagranie nie wychodzi do sieci | 1.4, 2.2, 3.2, 3.3 |
| VA-STT-2 | tylko GPU Metal, bez CPU | 3.1, 3.3, 5.5 |
| VA-REC-1 | ctrl+cmd+r start | 4.2, 5.4 |
| VA-REC-2 | ctrl+cmd+s stop + transkrypcja | 4.2, 4.3, 5.4 |
| VA-REC-3 | transkrypcja w schowku, cmd+v | 2.3, 4.1, 4.3 |
| VA-REC-4 | wybór mikrofonu | 1.3, 2.1, 5.3 |
| VA-MODEL-1 | model pobierany po instalacji, SHA-256, wznawianie, potem zero ruchu sieciowego | 3.2, 5.6, 6.1, 6.2 |
| VA-UI-1 | ikona szare/czerwone kółko | 5.1 |
| VA-UI-2 | klik ikony start/stop | 4.2, 5.2 |
| VA-HIST-1 | historia wypowiedzi: podmenu „Historia”, kopiowanie, zapis na dysku, wyczyść | 7.5, 7.6, 7.7 |
| VA-REC-5 | skracanie pauz wewnętrznych > 1,5 s do 0,5 s przed transkrypcją | 7.1, 7.2, 7.7 |
| VA-REC-6 | limit nagrania 600 s, auto-Stop z transkrypcją, powiadomienie | 7.3, 7.4, 7.7 |
| spec/APP_FLOW.md, spec/ux_ui/LINKS.md | niewypełnione szablony | poza zakresem |

## Notatki

- Zależności: `(wymaga: X.X, Y.Y)` - Ralph pominie zadanie jeśli zależności nie są ukończone
- Numeracja faz: cyfra lub cyfra+litera (np. `Faza 2`, `Faza 2M`, `Faza 6.Z1`). Suffix `.Z<n>` = faza rework wynikła ze zmiany w `changes/`.
- Dokumentacja: `(spec: plik.md, inny-plik.md)` - Ralph załaduje TYLKO powiązane fragmenty z tych plików (przez ralph/SPEC_INDEX.md + Read z offset/limit)
- Technologia: `(tech: tag1, tag2)` - Ralph załaduje wytyczne stylu/konwencji z `ralph-guidelines/<tag>.md` + zawsze `general.md`. Tagi muszą istnieć w `ralph-guidelines/` (brak = RALPH BLOCKED).
- Zmiana: `(zmiana: <nazwa-pliku>.md)` — adnotacja na zadaniach zmodyfikowanych lub utworzonych przez plik z `changes/`. Ralph przy realizacji wczyta `changes/processed/<nazwa-pliku>.md` żeby poznać kontekst (co i dlaczego zmienione).
- Każde zadanie implementacyjne ma adnotację `- [ ] Test: ...` (unit/integration) zaraz po sobie.
- Brak UI webowego/mobilnego → brak zadań e2e Playwright/Detox; przepływ end-to-end pokrywa test integracyjny z mockami (4.3) i test manualny ⛔ 6.4.
- Testy wymagające modelu, GPU, mikrofonu lub sesji graficznej oznaczaj `#[ignore]` z powodem; maszyna deweloperska (Mac Apple Silicon) jest zarazem platformą docelową — uruchamiaj je lokalnie `cargo test -- --include-ignored` przed końcem fazy.
- **Fazy rework `.Z<n>`**: tworzone automatycznie przez Ralpha gdy zmiana z `changes/` dotyka zadania ✅. Numeracja: `Faza <N>.Z1`, `.Z2`, ..., gdzie `N` to faza której zadanie jest reworkowane. Każde zadanie w fazie Z ma `(wymaga: <oryginalny-numer>)` na zadanie do przerobienia oraz `(zmiana: ...)`. Checkbox oryginalnego zadania NIE jest odznaczany — historia ukończeń pozostaje.
