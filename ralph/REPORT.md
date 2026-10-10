# ralph/REPORT.md - Raport realizacji projektu

<!-- Plik trafia W CAŁOŚCI do promptu startowego każdej sesji. Dwa rodzaje sekcji:            -->
<!--   • stan bieżący (Podsumowanie) — NADPISUJ, to ma być jeden aktualny opis;             -->
<!--   • kronika (Stan testów, Historia realizacji, Historia zmian, Log problemów,           -->
<!--     Artefakty) — DOPISUJ jeden wpis; starsze rotują się do ralph/REPORT_ARCHIVE.md      -->
<!--     przy starcie (Stan testów: 3 ostatnie przebiegi).                                   -->
<!-- Nie zakładaj własnych sekcji `## ` na podsumowania sesji — KAŻDA sekcja `## ` spoza      -->
<!-- tego szablonu jest doraźna i zostają z nich tylko 3 najnowsze.                          -->
<!-- Rotacja przenosi CAŁE wpisy, także wielolinijkowe. Wpis musi mieć rozpoznawalny         -->
<!-- początek (wiersz tabeli, `### …`, `- [data] …`, `- **Nazwa …**`), a akapit doklejony     -->
<!-- pod wpisem liczy się jako jego ciąg dalszy i pojedzie do archiwum razem z nim.           -->

## Podsumowanie

- **Start**: 2026-10-08
- **Status**: W trakcie — Faza 7 (historia wypowiedzi, pauzy, limit nagrania) dopisana 2026-10-10 z wymagań Specky VA-HIST-1, VA-REC-5, VA-REC-6
- **Postęp**: 47/60 pozycji ukończonych (Fazy 1–6 kompletne; Faza 7: 0/13)
- Specky: pracuję jako mariusz.iskra (mariusz.iskra@gmail.com), organizacja mariusz.iskra's Organization. Synchronizacja 2026-10-08 (kursor 704): 0 zmian, kolejka pusta; VA-MODEL-1 czeka na decyzję. Na prośbę właściciela VA-PLAT-1 i VA-TECH-1 oznaczone `code_ready`.
- Specky 2026-10-09 (kursor 709): kolejka pusta, VA-PLAT-2 `in_progress`.
- Otwarte PR: brak. 6.1 (b188bf1), 6.2 (3530198) i 6.3 (5f13dab) zmergowane; VA-PLAT-2 i VA-MODEL-1 `code_ready` (2026-10-09). Faza 5 zmergowana i otagowana (`ralph/faza-5`). `code_ready`: VA-STT-2, VA-REC-1..4, VA-UI-1, VA-UI-2. Wersja v0.2.0 utworzona (migawka Specky 01M4EVJ3FKXBKZG942NG0KYFVM); VA-STT-1 `code_ready`. Blokada kontraktu Specky rozwiązana decyzją właściciela (Specky nie obsługuje Rusta — kryteria ręcznie). VA-MODEL-1 zaakceptowane. Wersja v0.1.0 utworzona (migawka Specky 01M4EP1HG6SQR1GC25CTTSJ18P). Specky 2026-10-08 (kursor 705): kolejka pusta, VA-MODEL-1 czeka na decyzję właściciela.

## Stan testów

<!--
Tabela to stan bieżący — PODMIENIAJ liczby, nie dokładaj drugiej tabeli.
Nad tabelą jeden akapit per przebieg pełnego suite'u, otwarty pogrubieniem z datą:

**Regresja po Fazie 12 (2026-08-20) — ZIELONA:** backend `pytest -q` 340/0, frontend 210/0.
Względem 328/205 po Fazie 11: +12 testów backendu. Tag `ralph/faza-12`.

Data w pierwszej linii jest OBOWIĄZKOWA — po niej rotacja rozpoznaje, które przebiegi
zostawić (3 ostatnie), a które przenieść do ralph/REPORT_ARCHIVE.md. Akapit bez daty
jest traktowany jako ciąg dalszy przebiegu powyżej. Bez daty blok zostaje w pliku
na zawsze i rośnie w prompcie każdej sesji.
-->

**Regresja po Fazie 6 (2026-10-10) — ZIELONA:** na main (1ae141a) `cargo test --workspace -- --include-ignored` 126/0, clippy `-D warnings` czysto, `cargo fmt --check` czysto; test ręczny 6.4 właściciela (2026-10-09) bez uwag. Bez zmian liczby testów od przebiegu po 6.3. Tag `ralph/faza-6`.

**Przebieg po zadaniach 6.1–6.3 (2026-10-09) — ZIELONY (main 5f13dab, nie regresja fazy — czekało ⛔ 6.4):** `cargo test --workspace -- --include-ignored` 126/0 (w tym build release i bundle, montowanie .dmg, mikrofon, GPU Metal, model, schowek), clippy `-D warnings` czysto, `cargo fmt --check` czysto. Względem 119/0 po Fazie 5: +7 testów (bundle.rs 5, dmg.rs 2). Bez tagu `ralph/faza-6` do wyniku testu ręcznego 6.4.

**Regresja po Fazie 5 (2026-10-09) — ZIELONA:** na main (f76c960) `cargo test --workspace -- --include-ignored` 119/0 (z mikrofonem, GPU Metal, modelem i prawdziwym schowkiem), clippy `-D warnings` czysto, `cargo fmt --check` czysto. Względem 88/0 po Fazie 4: +31 testów. Tag `ralph/faza-5`. Zachowanie GUI (kliknięcia, skróty w innych aplikacjach, powiadomienia, wygląd ikony) poza zasięgiem testów automatycznych — scenariusze ręczne.

**Regresja po Fazie 4 (2026-10-09) — ZIELONA:** na main (96956d8) `cargo test --workspace -- --include-ignored` 88/0 (z mikrofonem, GPU Metal, modelem i prawdziwym schowkiem), clippy `-D warnings` czysto, `cargo fmt --check` czysto. Względem 70/0 po Fazie 3: +18 testów. Tag `ralph/faza-4`.

**Regresja po Fazie 3 (2026-10-09) — ZIELONA:** na main (ec17148) `cargo test --workspace -- --include-ignored` 70/0 (z testami mikrofonu, GPU Metal i prawdziwego modelu), clippy `-D warnings` czysto, `cargo fmt --check` czysto. Względem 49/0 po Fazie 2: +21 testów. Tag `ralph/faza-3`.

| Metryka | Wartość |
|---------|---------|
| Łącznie testów | 126 |
| Pass | 126 |
| Fail | 0 |
| Skip | 0 (z --include-ignored) |
| Ostatnie uruchomienie | 2026-10-10 (regresja Fazy 6 na main) |

## Historia realizacji

| Zadanie | Status | Testy | Próby | Commit | Czas |
|---------|--------|-------|-------|--------|------|
| 3.4 `va-dev transcribe` (WAV dowolny → 16 kHz mono → cisza → Whisper) + test 3.4 | ✅ PR #10 | 64/0 + 6 ignored; prawdziwa transkrypcja WAV 44,1 kHz stereo zielona lokalnie; mutacja normalizacji czerwona | 1 | ec17148 | 2026-10-09 |
| 4.1 Schowek: TextSink, ClipboardSink (reguła VA-REC-3), arboard + MemoryClipboard + test 4.1 | ✅ PR #13 | 68/0 + 7 ignored; prawdziwy schowek zielony lokalnie; mutacja reguły pustej transkrypcji czerwona | 1 | 45d1f38 | 2026-10-09 |
| 4.2 Automat stanów (StateMachine, Input/Action/Transition) + test 4.2 (tablica przejść) | ✅ PR #15 | 75/0 + 7 ignored; mutacja Start w Recording czerwona | 1 | fc2852e | 2026-10-09 |
| 4.3 Kontroler (wątek kontrolera + wątek transkrypcji, zdarzenia dla UI) + test 4.3 | ✅ PR #16 | 81/0 + 7 ignored; 3× powtórzony bez flaków; mutacja pomijania ciszy czerwona | 1 | 96956d8 | 2026-10-09 |
| 5.1 Aplikacja paska menu (tao + tray-icon, Accessory), ikona stanu rysowana w kodzie + test 5.1 | ✅ PR #18 | 86/0 + 7 ignored; aplikacja startuje lokalnie (GPU, model, pętla zdarzeń); mutacja koloru Recording czerwona | 1 | 7559763 | 2026-10-09 |
| 5.2 Kliknięcie ikony → Start/Stop (click::command_for, zdarzenia tray-icon w pętli tao) + test 5.2 | ✅ PR #19 | 91/0 + 7 ignored; mutacja czerwone→Start czerwona | 1 | 9a161ae | 2026-10-09 |
| 5.3 Menu: podmenu Mikrofon (zaznaczony używany, zapis do konfiguracji), Zakończ + test 5.3 | ✅ PR #20 | 96/0 + 7 ignored; mutacja zaznaczenia czerwona; aplikacja z menu startuje lokalnie | 1 | d3a22a4 | 2026-10-09 |
| 5.4 Skróty globalne ctrl+cmd+r / ctrl+cmd+s (global-hotkey), komunikat w menu przy konflikcie + test 5.4 | ✅ PR #21 | 100/0 + 7 ignored; rejestracja skrótów lokalnie bez błędu; mutacja zamiany Start/Stop czerwona | 1 | b7458d8 | 2026-10-09 |
| 5.5 Komunikaty: cisza cyfrowa → NoSignal + powiadomienie, brak GPU/modelu → menu + Start z powodem + test 5.5 | ✅ PR #22 | 106/0 + 7 ignored; mutacja wykrywania ciszy cyfrowej czerwona; start bez modelu lokalnie OK | 1 | 2607ff7 | 2026-10-09 |
| 5.6 Pobieranie modelu w tle (DownloadState, status i „Ponów” w menu, kontroler po pobraniu) + test 5.6 | ✅ PR #23 | 112/0 + 7 ignored; mutacja blokady Startu czerwona; start bez modelu pobiera w tle (121 MB/8 s) | 1 | f76c960 | 2026-10-09 |
| 6.1 Bundle VoiceAsystent.app (`scripts/build-app.sh`, Info.plist, ikona z `scripts/app-icon.py`, podpis ad-hoc) + test 6.1 | ✅ PR #26 | 116/0 + 8 ignored; build release + bundle 8,4 MB + start przez `open` z załadowaniem modelu na Metalu lokalnie OK; mutacja klucza NSMicrophoneUsageDescription czerwona | 1 | b188bf1 | 2026-10-09 |
| 6.2 `scripts/build-dmg.sh` (hdiutil UDZO, aplikacja + skrót do Applications, wersja z git describe) + test 6.2 | ✅ PR #27 | 118/0 + 8 ignored; obraz release 3,9 MB montuje się z aplikacją i skrótem; mutacja usunięcia skrótu czerwona | 1 | 3530198 | 2026-10-09 |
| 6.3 README.md + docs/RUNBOOK.md (wymagania, instalacja z .dmg i Gatekeeper, pierwsze uruchomienie, skróty, komendy, diagnostyka) | ✅ PR #28 | bez testów (dokumentacja); twierdzenia zweryfikowane w kodzie (limit nagrania, `.part`, SHA-256) | 1 | 5f13dab | 2026-10-09 |
| 6.4 Test manualny właściciela (instalacja z .dmg, nagranie skrótami i kliknięciem, cmd+v w kilku aplikacjach, zmiana mikrofonu) | ✅ ręcznie | właściciel 2026-10-09 wg docs/test-scenarios/v0.4.0.md: wszystkie scenariusze zgodne z oczekiwaniami, bariery odrzucone; 2.2 (macOS 13–14) nie do sprawdzenia | 1 | — | 2026-10-10 |
| 7.1 `compress_pauses` w va-audio (pauzy > 1,5 s → 0,5 s, mowa i brzegi bez zmian) + test 7.1 | ✅ PR #33 | 124/0 + 8 ignored; mutacja progu 1,5 s → 0,5 s czerwona (2 testy) | 1 | 3f44694 | 2026-10-10 |
| 7.2 Wpięcie `compress_pauses` w kontrolerze i `va-dev transcribe`, fixture `speech_pl_long_pause.wav` (2 zdania + 40 s ciszy) + test 7.2 | PR #34 otwarty | 126/0 + 9 ignored; prawdziwa transkrypcja fixture: oba zdania, 17 słów, nic między nimi; mutacja (pominięcie compress_pauses w kontrolerze) czerwona | 1 | — | 2026-10-10 |

## Historia zmian

<!--
Wpisy dodawane przez Ralpha po przetworzeniu pliku z changes/.
Format wpisu (jeden ### na każdą zmianę):

### YYYY-MM-DD <nazwa-pliku>.md
- **Rodzaj**: błąd | korekta | rozwój
- **Źródło błędu**: [tylko przy błędzie: zadanie X.Y, które to zbudowało | nieznane]
- **Wykryte przez**: [tylko przy błędzie: test | scenariusz | review | ralph | właściciel | produkcja]
- **Priorytet**: krytyczny | normalny
- **Dotyczy**: [pole dotyczy z frontmattera lub "ogólne"]
- **Streszczenie**: [1-2 zdania co się zmienia i dlaczego]
- **Zmodyfikowane zadania pending**: [lista numerów, np. 4.2, 5.1] lub "brak"
- **Nowa faza rework**: [np. Faza 6.Z1 z 3 zadaniami] lub "brak"
- **Nowe zadania**: [lista numerów, np. 7.4, 7.5] lub "brak"

Plik źródłowy zmiany żyje w changes/processed/<nazwa-pliku>.md (audyt w gicie).
-->

### 2026-10-10 2026-10-10-specky-CY0SQA5.md
- **Rodzaj**: rozwój
- **Priorytet**: normalny
- **Dotyczy**: ogólne (nowe wymaganie Specky VA-HIST-1, sekcja „Nagrywanie i schowek”)
- **Streszczenie**: Historia wypowiedzi: każda niepusta transkrypcja trafia do trwałej listy (30 wpisów, plik w katalogu danych aplikacji), podmenu „Historia” pozwala skopiować wcześniejszy tekst do schowka i wyczyścić listę; decyzja właściciela 2026-10-10.
- **Zmodyfikowane zadania pending**: brak
- **Nowa faza rework**: brak
- **Nowe zadania**: 7.5, 7.6, 7.7 (Faza 7)

### 2026-10-10 2026-10-10-specky-1BY4F4Z.md
- **Rodzaj**: rozwój
- **Priorytet**: normalny
- **Dotyczy**: ogólne (nowe wymaganie Specky VA-REC-5; crates/audio/src/silence.rs, kontroler)
- **Streszczenie**: Pauzy wewnątrz nagrania dłuższe niż 1,5 s są skracane do 0,5 s przed transkrypcją, żeby Whisper nie halucynował w oknach bez mowy; brzegi jak dotąd.
- **Zmodyfikowane zadania pending**: brak
- **Nowa faza rework**: brak
- **Nowe zadania**: 7.1, 7.2 (Faza 7)

### 2026-10-10 2026-10-10-specky-9VFYND0.md
- **Rodzaj**: rozwój
- **Priorytet**: normalny
- **Dotyczy**: ogólne (nowe wymaganie Specky VA-REC-6; recorder, va-config, kontroler, messages.rs)
- **Streszczenie**: Limit nagrania domyślnie 600 s; po jego osiągnięciu nagrywarka zgłasza limit, kontroler kończy nagranie jak po ctrl+cmd+s (transkrypcja do schowka), a użytkownik dostaje powiadomienie — dziś nadwyżka ginęła po cichu.
- **Zmodyfikowane zadania pending**: brak
- **Nowa faza rework**: brak
- **Nowe zadania**: 7.3, 7.4 (Faza 7)

## Artefakty wizualne

<!--
Sekcja uzupełniana TYLKO gdy w ralph/config.md sekcja `## Artefakty wizualne` ma
`screeny: tak` lub `nagrania: tak`. Pełna procedura: sekcja 0.5 instrukcji.

Format wiersza:
| Faza | Typ | ID | Plik artefaktu | Spec source |
|------|-----|----|--------------:|-------------|
| 1    | screen | B1_ProjectOverview | artifacts/screens/B1_ProjectOverview.png | spec/ux_ui/screens-b1-b2.jsx:52-200 |
| 1    | flow   | krok-1             | artifacts/flows/krok-1-rejestracja.webm  | spec/APP_FLOW.md:11-30 |

Info-warningi (brak APP_FLOW.md mimo nagrania:tak, brak flow_files, itp.) zapisuj
poniżej tabeli z prefiksem `> ⚠ Info:` tak żeby user widział co poszło "po cichu".
-->

| Faza | Typ | ID | Plik artefaktu | Spec source |
|------|-----|----|----------------|-------------|
| | | | | |

## Log problemów

<!-- Format: [data] Zadanie X.X: opis problemu - rozwiązanie/status -->
<!-- Listy zmienionych plików NIE prowadzimy — git zna ją lepiej: git log --stat -->
- [2026-10-10] Zadanie 7.2: zmiana testu `transcribe_keeps_both_sentences_around_a_long_pause_without_extra_phrases` — słowo kluczowe «biurze» → «jutro»: syntezator `say` (Zosia) wymawia „biurze” tak, że Whisper daje „Białże”; kryterium 01M4K06AR8TJXY24K05X0MNBJC (VA-REC-5, changes/processed/2026-10-10-specky-1BY4F4Z.md) wymaga obu zdań bez dodatkowych fraz, nie rozpoznania konkretnego słowa. Wynik prawdziwej transkrypcji: „Dzisiaj jest piękna pogoda, idę na spacer do parku. Jutro rano mam spotkanie w Białże o 9.00.” — 17 słów, bez halucynacji w 40 s pauzy.
- [2026-10-10] Start Fazy 7: pełny suite po przetworzeniu changes/ nie powtórzony — kod (crates/apps/scripts/Cargo) bajt w bajt ten sam, co w regresji Fazy 6 z 2026-10-10 (126/0); zmiany dotyczyły tylko ralph/, docs/ i changes/.
- [2026-10-09] Zadanie 6.1: odstępstwo od planu — skrypt nie przekazuje `--features metal`: feature `metal` jest włączony na stałe w zależności `va-stt` binarki, więc `cargo build --release -p voice-asystent` daje build z Metalem (potwierdzone logiem `ggml_metal_library_init: using embedded metal library` po starcie z bundla).
- [2026-10-09] Zadanie 6.1: ostrzeżenie kontroli `testy/skip-z-powodem` — test `release_build_creates_dist_app` ma `#[ignore]` (build release trwa minuty), uruchamiany w pełnym suicie `--include-ignored`; zielony lokalnie (40 s przy ciepłym cache). Pozostałe 4 testy bundla biegną w CI z binarki debug (`--binary`).
- [2026-10-09] Zadanie 6.1: wygląd ikony aplikacji w Finderze i nazwa aplikacji w powiadomieniach niezweryfikowane wizualnie — test ręczny 6.4. LSMinimumSystemVersion ustawione na 13.0 (brak wymagania w spec; whisper.cpp Metal i tao wymagają nowoczesnego macOS).
- [2026-10-09] Zadanie 5.1: zmiana testów startup_reports_gpu_verdict / startup_reports_missing_model — dodany argument `--self-check` (asercje bez zmian). Powód: VA-UI-1 (Specky 01M4EKHD1BQYRNWBAVRA5FJTRC) wymaga, by aplikacja trwała w pasku menu, więc bez flagi proces nie kończy się i test by wisiał.
- [2026-10-09] Zadanie 5.1: odstępstwo od planu — ikony nie jako pliki PNG @1x/@2x, tylko RGBA 36 px rysowane w kodzie (ten sam efekt na Retinie, bez binarnych zasobów, kolor sprawdzalny testem). Wygląd w pasku menu niezweryfikowany wizualnie (brak uprawnienia do nagrywania ekranu) — do testu ręcznego.
- [2026-10-09] Zadanie 5.1: start aplikacji liczy SHA-256 modelu ~3,5 s przed pokazaniem ikony — do rozważenia przy 5.6 (np. sprawdzanie w tle).
- [2026-10-09] Zadanie 4.3: AC «wejście — polecenia Start/Stop z 5.2 i 5.4» — wejście kliknięciem podpięte w 5.2 (`app.rs` → `controller.send`), skróty w 5.4.
- [2026-10-09] Zadanie 5.5: odczyt statusu zgody na mikrofon (AVCaptureDevice.authorizationStatus) wymaga `unsafe` (metoda może rzucić NSException), a workspace ma `unsafe_code = deny` — zamiast tego wykrywanie ciszy cyfrowej (same zera po Stop), którą macOS podaje bez zgody. Uwaga: wirtualne wejście (np. BlackHole) bez dźwięku też da ten komunikat. Powiadomienia przez mac-notification-sys (bez bundla pokazują się jako aplikacja Terminal — do weryfikacji po 6.1).
- [2026-10-09] Zadanie 5.5: brak miejsca na dysku w trakcie testów (256 MB wolnego) — wyczyszczone artefakty buildu (`cargo clean` dla katalogu mutacji w scratchpadzie i `--release`), wolne 8,3 GB; testy powtórzone, zielone.
- [2026-10-09] Zadanie 5.4: naciśnięcie skrótu w innej aplikacji niezweryfikowane automatycznie (symulacja klawiszy wymagałaby uprawnienia Dostępność dla terminala) — do testu ręcznego; ostrzeżenie `kod-bez-testu` jak w 5.2.
- [2026-10-09] Zadanie 5.3: lista mikrofonów odświeżana przy najechaniu na ikonę (TrayIconEvent::Enter) i prawym kliknięciu — obsługa idzie przez pętlę zdarzeń asynchronicznie, więc menu otwarte bez wcześniejszego najechania może pokazać listę sprzed chwili; do sprawdzenia ręcznie (podłączenie słuchawek). Ostrzeżenie `kod-bez-testu` jak w 5.2.
- [2026-10-09] Zadanie 5.2: ostrzeżenie kontroli `testy/kod-bez-testu` — testy Rusta są w tym samym pliku (`#[cfg(test)] mod tests` w click.rs), czego kontrola nie widzi; podpięcie w app.rs (pętla tao) bez testu automatycznego — sprawdzi test ręczny (scenariusz po Fazie 5).
- [2026-10-09] [check] PR #9 — Ralph: kontrole (lokalnie): kontrola `zaleznosci` nie wykonała się (UnicodeDecodeError) — `ralph-kontrole/zaleznosci/run.py:311-315` czyta każdy zmieniony plik jako UTF-8 przed sprawdzeniem, czy to manifest, i łapie tylko OSError; binarne WAV z fixtures ją wywracają (też przy tagu ralph/faza-2 i v0.1.0). Błąd frameworka, nie kodu — RALPH BLOCKED; 2026-10-09 właściciel naprawił kontrolę (pomija pliki niebędące manifestem) i PATH hooka (~/.cargo/bin), blokada usunięta.
