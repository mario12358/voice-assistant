# DECISIONS.md — Changelog of interactive-mode decisions

<!--
Created and filled automatically by Claude in interactive mode
(after TELESMA COMPLETE / TELESMA BLOCKED, when the user drives the conversation manually).

Every user↔Claude exchange in this mode = one entry below.
Entry format (section 10 of the instructions):

## YYYY-MM-DD HH:MM — [short topic]
**Kind**: bug | correction | feature | question | operations
**Bug source**: [bugs only: task X.Y that built it | unknown]
**Detected by**: [bugs only: test | scenario | review | telesma | owner | production]
**Question/context**: [paraphrase of what the user asked, 1-2 sentences]
**Outcome**: [essence of the decision/explanation, 2-4 sentences, no code]
**Impact** (optional): [task X.Y / file / phase]

This file is WRITE-ONLY — Ralph does not read it in any other process.
It is a changelog for humans, to remember "what we agreed on and why".
-->


## 2026-10-08 22:28 — Tylko macOS, Specky źródłem wymagań
**Kind**: correction
**Question/context**: Właściciel odpowiedział na TELESMA BLOCKED. Aplikacja ma działać wyłącznie na macOS i być instalowana z pliku instalacyjnego. Specky ma być jedynym źródłem wymagań. Repozytorium git z origin już istnieje.
**Outcome**: Plan przepisany od zera pod aplikację paska menu macOS: 6 faz, 45 pozycji, GPU Metal, schowek zamiast wpisywania tekstu, bez LLM i TTS, instalator .dmg. 11 wymagań (wymagania.md + WYTYCZNE_TECHNICZNE.md) zaproponowano w Specky jako wsad 01M4EK43A6GBPR63H0FV0VBCV0 do akceptacji właściciela. W wymaganiu 3 („ctrl + v”) przyjęto cmd + v ze wstępu wymagania.md.
**Impact**: telesma/state/PLAN.md (cały), Specky (11 propozycji)

## 2026-10-08 22:36 — Akceptacja wymagań, model pobierany po instalacji
**Kind**: feature
**Question/context**: Właściciel zatwierdził wsad wymagań w Specky, zainstalował Rust i zdecydował, że model ma być pobierany po instalacji aplikacji.
**Outcome**: 11 wymagań zaakceptowanych w Specky i podpiętych do planu jako kotwice. Model nie trafia do .dmg: aplikacja pobiera go przy pierwszym uruchomieniu (z postępem, wznawianiem i sumą SHA-256). Dodano zadanie 5.6 i propozycję wymagania VA-MODEL-1 do akceptacji.
**Impact**: zadania 3.2, 5.6, 6.1, 6.2, 6.3

## 2026-10-08 22:43 — Bez wydania po Fazie 1, stan Specky
**Kind**: question
**Question/context**: Właściciel odrzucił wydanie v0.1.0 po Fazie 1, kazał kontynuować i zapytał, czy w Specky coś czeka.
**Outcome**: Brak wersji, praca idzie dalej od zadania 2.1. W Specky czeka na decyzję tylko propozycja VA-MODEL-1. Kolejka poprawek jest pusta; 3 wymagania są w realizacji, 8 otwartych. Kryteria nie dostaną dowodów z testów, dopóki nie ma CI (integracja z repozytorium w config: brak).
**Impact**: Faza 2

## 2026-10-08 22:58 — Pauza po zadaniu 2.2
**Kind**: operations
**Question/context**: W trakcie pracy właściciel poprosił o zatrzymanie po bieżącym zadaniu z planu.
**Outcome**: Zadanie 2.2 zostało dokończone i zacommitowane, potem praca stanęła. Następne do zrobienia jest 2.3 (przycinanie ciszy).
**Impact**: Faza 2

## 2026-10-08 23:10 — code_ready dla VA-PLAT-1 i VA-TECH-1
**Kind**: operations
**Question/context**: Właściciel w trakcie zadania 2.3 poprosił o ustawienie w Specky statusu code_ready dla VA-PLAT-1 i VA-TECH-1.
**Outcome**: Ustawiono code_ready dla obu wymagań (odcisk treści v1 z get_requirement). Faza 1 (szkielet macOS-only, workspace Cargo) je realizuje; status done ustala właściciel.
**Impact**: Specky — VA-PLAT-1, VA-TECH-1

## 2026-10-08 23:17 — Wydanie wersji v0.1.0
**Kind**: operations
**Question/context**: Właściciel przyjął propozycję wydania po zielonej regresji Fazy 2.
**Outcome**: Utworzono wersję v0.1.0 (commit 57cf6af) obejmującą fazy 1–2: szkielet macOS, konfigurację, wybór mikrofonu, nagrywanie do 16 kHz mono i przycinanie ciszy. Scenariusze testów w docs/test-scenarios/v0.1.0.md, migawka w Specky 01M4EP1HG6SQR1GC25CTTSJ18P. Wersja jest tylko lokalnie do czasu git push origin v0.1.0.
**Impact**: tag v0.1.0, Specky

## 2026-10-08 23:20 — cargo niedostępny w terminalu właściciela
**Kind**: question
**Question/context**: Przy scenariuszach v0.1.0 terminal zgłaszał „command not found: cargo”.
**Outcome**: Rust jest zainstalowany, a ~/.zshenv ładuje ~/.cargo/env; terminal był otwarty przed instalacją. Wystarczy nowy terminal albo `source "$HOME/.cargo/env"`. Hook kontroli to osobna sprawa (nie czyta ~/.zshenv) — do sprawdzenia.
**Impact**: docs/test-scenarios/v0.1.0.md (sekcja 0)

## 2026-10-08 23:24 — Mikrofon z konfiguracji nie ustawia się
**Kind**: question
**Question/context**: Przy scenariuszu 1.2 v0.1.0 wybór mikrofonu w config.toml nie działał.
**Outcome**: Plik ~/Library/Application Support/VoiceAsystent/config.toml (ani katalog) nie istnieje, więc działają wartości domyślne — kod zachowuje się poprawnie. Podano dokładne nazwy urządzeń (PXC 550, BlackHole 2ch, Mikrofon (MacBook Air)) i sposób utworzenia pliku; klucz `microphone` musi stać przed sekcją [silence].
**Impact**: docs/test-scenarios/v0.1.0.md (scenariusz 1.2)

## 2026-10-08 23:26 — Ustawienie mikrofonu PXC 550
**Kind**: operations
**Question/context**: Właściciel wybrał PXC 550 jako mikrofon do testów v0.1.0.
**Outcome**: Utworzono ~/Library/Application Support/VoiceAsystent/config.toml z `microphone = "PXC 550"`; `va-dev devices` pokazuje `>` przy PXC 550, pozostałe ustawienia domyślne.
**Impact**: scenariusz 1.2 v0.1.0

## 2026-10-08 23:44 — Wymagania czekające na akceptację w Specky
**Kind**: question
**Question/context**: Właściciel zapytał, czy w Specky są wymagania do zaakceptowania.
**Outcome**: Czeka jedna propozycja: VA-MODEL-1 (model pobierany po instalacji, wsad 01M4EKHX2XBS6ZSPSE9DAV9QFD, zmiana 01M4EKHXE3M2312B7EECQ55EHJ). Od niej zależą 3.2 i 5.6. Brak otwartych pytań i kolizji. Akceptacja na wyraźną prośbę właściciela.
**Impact**: zadania 3.2, 5.6

## 2026-10-08 23:47 — Akceptacja VA-MODEL-1
**Kind**: feature
**Question/context**: Właściciel poprosił o zaakceptowanie propozycji VA-MODEL-1 w Specky.
**Outcome**: Zaakceptowano (wymaganie 01M4EQH9D30871ZG5JMWK8VHFV v1, 6 kryteriów: .dmg bez modelu, pobieranie z postępem przy pierwszym starcie, SHA-256 z usunięciem uszkodzonego pliku, wznawianie, komunikat zamiast nagrania przed pobraniem, zero ruchu sieciowego po pobraniu). Kotwice dopisane do 3.2, 5.6, 6.1.
**Impact**: zadania 3.2, 5.6, 6.1, 6.2; telesma/state/SPECKY.md

## 2026-10-08 23:52 — Kryteria Specky bez dowodów z CI (Rust)
**Kind**: operations
**Question/context**: Rozstrzygnięcie blokady: Specky nie łączył testów Rusta ze znacznikami kryteriów.
**Outcome**: Właściciel wyjaśnił, że Specky czyta znaczniki tylko w .py/.js/.ts. Ustaliliśmy wariant (c): pracujemy dalej bez dowodów z CI, znaczniki `specky: crit` w testach zostają na przyszłość, kryteria właściciel odhacza ręcznie w komentarzu Specky na PR. Blokada usunięta.
**Impact**: sekcja 18.5 kontraktu CI; telesma/state/BLOCKED.md usunięty; PROJECT_CONTEXT (📌)

## 2026-10-09 00:40 — Naprawa kontroli zaleznosci i PATH hooka
**Kind**: bug
**Bug source**: unknown
**Detected by**: telesma
**Question/context**: Właściciel naprawił kontrolę `zaleznosci` (wywracała się na binarnych WAV) i PATH hooka (brak ~/.cargo/bin), prosząc o dołączenie poprawek do PR #9 i merge.
**Outcome**: Poprawki dołączone do gałęzi PR #9; amend ostatniego commita nie wystarczył (dowód dla wcześniejszego commita wciąż „nie wykonała się”), więc oba commity gałęzi scalone `reset --soft` i zacommitowane ponownie przez hook — wszystkie 11 kontroli przeszło, w tym clippy/fmt. PR #9 zmergowany, blokada usunięta.
**Impact**: zadanie 3.3, ralph-kontrole/zaleznosci/run.py, ralph-kontrole/ralph-kontrole.py

## 2026-10-09 00:55 — Wydanie wersji v0.2.0
**Kind**: operations
**Question/context**: Właściciel przyjął propozycję wydania po zielonej regresji Fazy 3.
**Outcome**: Utworzono wersję v0.2.0 (commit e338ac8): sprawdzanie GPU Metal bez fallbacku CPU, pobieranie modelu z wznawianiem i SHA-256, transkrypcja WAV na GPU (`va-dev transcribe`). Scenariusze w docs/test-scenarios/v0.2.0.md, migawka Specky 01M4EVJ3FKXBKZG942NG0KYFVM. Wersja lokalnie do czasu git push origin v0.2.0.
**Impact**: tag v0.2.0, Specky

## 2026-10-09 01:00 — Pauza po zadaniu 4.1
**Kind**: operations
**Question/context**: W trakcie finalizacji 4.1 właściciel poprosił o dokończenie zadania i zatrzymanie pracy.
**Outcome**: Zadanie 4.1 (schowek) zmergowane (PR #13) i odhaczone, praca zatrzymana. Następne do zrobienia jest 4.2 (automat stanów nagrywania).
**Impact**: Faza 4

## 2026-10-09 08:30 — Bez wydania po Fazie 4
**Kind**: operations
**Question/context**: Propozycja wydania v0.3.0 po zielonej regresji Fazy 4 (zmiany bez widocznego efektu dla użytkownika).
**Outcome**: Właściciel odrzucił wydanie zgodnie z rekomendacją — następna propozycja po Fazie 5 (ikona, skróty, schowek w działającej aplikacji). Praca idzie dalej od 5.1.
**Impact**: Faza 5

## 2026-10-09 09:52 — Wydanie wersji v0.3.0
**Kind**: operations
**Question/context**: Właściciel przyjął propozycję wydania po zielonej regresji Fazy 5.
**Outcome**: Utworzono wersję v0.3.0 (commit 23ec3de) obejmującą fazy 4–5: działający asystent w pasku menu (kliknięcie i skróty, schowek, menu mikrofonu, komunikaty, pobieranie modelu w tle). Scenariusze w docs/test-scenarios/v0.3.0.md, migawka Specky 01M4FTCC8WGF4QWM98D0CE7JEG. Wersja lokalnie do czasu git push origin v0.3.0.
**Impact**: tag v0.3.0, Specky

## 2026-10-09 11:15 — Wydanie wersji v0.4.0
**Kind**: operations
**Question/context**: Właściciel przyjął propozycję wydania po zielonym pełnym suicie po zadaniach 6.1–6.3 (instalator).
**Outcome**: Utworzono wersję v0.4.0 (commit 1ae141a): bundle VoiceAsystent.app z własną ikoną i polskim opisem uprawnienia mikrofonu, obraz .dmg z przeciąganiem do Applications, README i RUNBOOK. Scenariusze w docs/test-scenarios/v0.4.0.md, migawka Specky 01M4FZ5ZY6SRS6JTDZKJQNVK28. Wersja lokalnie do czasu git push origin v0.4.0. Test ręczny 6.4 nadal czeka na właściciela — Faza 6 bez tagu fazy.
**Impact**: tag v0.4.0, Specky, zadanie 6.4

## 2026-10-09 12:33 — Stan Specky po wydaniu v0.4.0
**Kind**: question
**Question/context**: Właściciel zapytał, co nowego w Specky.
**Outcome**: Od kursora 709 jedyna nowa pozycja to migawka v0.4.0 (01M4FZ5ZY6SRS6JTDZKJQNVK28). Kolejka pracy pusta: 0 do poprawy, 0 do zbudowania, 8 wymagań code_ready, 4 done (jedno z ostatnio oznaczonych code_ready właściciel przeniósł na done). Brak dryfu, brak czekających propozycji, brak czerwonych PR-ów.
**Impact**: brak

## 2026-10-09 12:34 — Ponowne sprawdzenie nowości
**Kind**: question
**Question/context**: Właściciel zapytał, czy jest coś nowego (bez wskazania kanału).
**Outcome**: Sprawdzone wszystkie kanały: Specky bez nowych pozycji od kursora 710, telesma/changes/ zawiera tylko szablon PRZYKLAD.md (bez zmian do przetworzenia), brak otwartych PR-ów, main równe z origin/main. Jedyne, co czeka: wersja v0.4.0 nie jest jeszcze wypchnięta (git push origin v0.4.0) i test ręczny 6.4.
**Impact**: brak

## 2026-10-09 12:35 — Nowe wymaganie: wsparcie Windows
**Kind**: question
**Question/context**: Właściciel dodał przez chat Specky wymaganie „Aplikacja ma wspierać również Windows” (01M4G3NMKKEBCK15MKCVXN4789, zaakceptowane, 3 kryteria) i zapytał o nie.
**Outcome**: Wymaganie koliduje z VA-PLAT-1 (wyłącznie macOS, status done, kryterium „brak konfiguracji budowania dla Windows”), VA-STT-2 (tylko GPU Metal) i VA-PLAT-2 (tylko .dmg); kryterium 2 mówi o „rozmowie z LLM”, której w projekcie nie ma (decyzja 2026-10-08: bez LLM). Przedstawiono właścicielowi listę rozstrzygnięć do podjęcia w Specky (zastąpienie VA-PLAT-1, GPU na Windows: CUDA/Vulkan, skróty i schowek na Windows, LLM tak/nie, external_ref i sekcja) oraz szacunek: osobna faza, bez maszyny z Windows nieweryfikowalna. Bez zmian w planie do decyzji właściciela.
**Impact**: VA-PLAT-1, VA-STT-2, VA-PLAT-2; przyszła Faza 7

## 2026-10-09 22:34 — Cofnięcie wymagania o Windows w Specky
**Kind**: correction
**Question/context**: Właściciel polecił cofnąć w Specky obsługę Windows i Linux — projekt ma zostać wyłącznie na macOS.
**Outcome**: Wymaganie 01M4G3NMKKEBCK15MKCVXN4789 („wspierać również Windows”) wycofane (deprecate bez następcy): propozycja 01M4H5YZBGV79B02KW3B4T66NJ zaakceptowana na wyraźną prośbę właściciela. VA-PLAT-1 (wyłącznie macOS) obowiązuje bez zmian; osobnego wymagania o Linux nigdy nie było. Plan bez zmian — Faza 7 nie powstaje.
**Impact**: Specky; decyzja 📌 „wyłącznie macOS” w PROJECT_CONTEXT pozostaje

## 2026-10-10 12:07 — Historia wypowiedzi (pomysł na nową funkcję)
**Kind**: question
**Question/context**: Właściciel chce mieć „schowek na wypowiedzi” — możliwość wrócenia do wcześniejszych transkrypcji i skopiowania ich ponownie; pyta, jak do tego podejść.
**Outcome**: Rekomendacja: podmenu „Historia” pod prawym kliknięciem ikony (ostatnie N wpisów, skrót tekstu + godzina, kliknięcie kopiuje pełny tekst do schowka, pozycja „Wyczyść historię”); logika w va-core (bufor po zdarzeniu Delivered), zapis na dysk w katalogu danych aplikacji z limitem wpisów — wymaga decyzji właściciela, bo to pierwsze trwałe przechowywanie treści (dotąd tylko schowek). Ścieżka formalna: nowe wymaganie w Specky (np. VA-HIST-1) → telesma/changes/ → Faza 7 (ok. 4–5 zadań). Na razie bez zmian w planie.
**Impact**: przyszła Faza 7; decyzja o trwałości i prywatności historii

## 2026-10-10 15:24 — Historia wypowiedzi zapisywana na dysku; limit i pauzy w nagraniu
**Kind**: feature
**Question/context**: Właściciel zdecydował, że historia wypowiedzi ma być zapisywana na dysku, i zapytał, jak długie mogą być nagrania oraz czy 5-minutowe nagranie z długimi przerwami w mówieniu zostanie obsłużone.
**Outcome**: Zaproponowano w Specky wymaganie VA-HIST-1 (podmenu „Historia”, kliknięcie kopiuje tekst, limit 30 wpisów, zapis w katalogu danych aplikacji, „Wyczyść historię”, treść poza logami) — czeka na akceptację właściciela, potem telesma/changes/ → Faza 7. Nagrania: limit 5 min (max_recording_secs=300, zmienny w config.toml); dokładnie 5 min mieści się, nadwyżka jest po cichu odrzucana (tylko ostrzeżenie w logu, ikona dalej czerwona) — luka do naprawy. Cisza jest przycinana tylko na brzegach; długie pauzy w środku idą do Whispera i grożą halucynacjami w oknach 30 s bez mowy. Zaproponowano: kompresję pauz wewnętrznych przed transkrypcją i powiadomienie/auto-stop przy limicie — do decyzji właściciela.
**Impact**: Specky VA-HIST-1; przyszła Faza 7; crates/audio (pauzy), recorder (limit)

## 2026-10-10 15:29 — Propozycje VA-REC-5 i VA-REC-6 w Specky
**Kind**: feature
**Question/context**: Właściciel polecił wystawić przez Specky dwa wymagania zaproponowane przy analizie długich nagrań.
**Outcome**: Zaproponowano w tym samym wsadzie co VA-HIST-1: VA-REC-5 (skracanie pauz wewnętrznych > 1,5 s do 0,5 s przed transkrypcją, 4 kryteria) i VA-REC-6 (limit nagrania domyślnie 10 min, automatyczny Stop z transkrypcją i powiadomienie przy limicie, 4 kryteria). Wsad 01M4JZS73RFCBAXE91TSTJCG2V czeka na akceptację właściciela; po niej następny start Ralpha założy pliki w telesma/changes/ i Fazę 7.
**Impact**: Specky; przyszła Faza 7 (crates/audio, recorder, kontroler, powiadomienia)

## 2026-10-10 15:32 — Akceptacja VA-HIST-1, VA-REC-5, VA-REC-6; pytanie o test ręczny
**Kind**: feature
**Question/context**: Właściciel potwierdził treść trzech propozycji i polecił je zaakceptować w Specky; push wersji v0.4.0 zostawia na później; zapytał, czym jest test ręczny.
**Outcome**: Wsad 01M4JZS73RFCBAXE91TSTJCG2V zaakceptowany na wyraźną prośbę właściciela: VA-HIST-1 (01M4K06AGAV8G79RJMXCY0SQA5), VA-REC-5 (01M4K06AQB7B8055HQ21BY4F4Z), VA-REC-6 (01M4K06AXGNSZ9XEZ3E9VFYND0). Przegląd sprzeczności z resztą projektu: brak (VA-REC-6 uzupełnia VA-REC-2, VA-REC-5 zachowuje VA-REC-3). Mapa w telesma/state/SPECKY.md uzupełniona; następny start Ralpha założy telesma/changes/ i Fazę 7. Test ręczny = zadanie ⛔ 6.4 z planu wg docs/test-scenarios/v0.4.0.md.
**Impact**: Specky; Faza 7; zadanie 6.4

## 2026-10-10 15:34 — Wynik testu ręcznego 6.4: wszystko OK
**Kind**: operations
**Question/context**: Właściciel zgłosił, że test ręczny 6.4 (instalacja z .dmg, nagrywanie, wklejanie, zmiana mikrofonu) wykonał 2026-10-09 i wszystko działa.
**Outcome**: Zadanie 6.4 odhaczone, wyniki wpisane do docs/test-scenarios/v0.4.0.md (wszystkie scenariusze [x], 2.2 nie do sprawdzenia na dev, decyzja: gotowa do wdrożenia) i do REPORT.md. Plan 47/47. Regresja Fazy 6 na main uruchomiona; po zieleni tag telesma/phase-6 i PR stanu. Push wersji v0.4.0 nadal po stronie właściciela.
**Impact**: Faza 6 zamknięta; tag telesma/phase-6

## 2026-10-10 17:08 — Wydanie wersji v0.5.0
**Kind**: operations
**Question/context**: Właściciel przyjął propozycję wydania po zielonym pełnym suicie po zadaniach 7.1–7.6 (test ręczny 7.7 jeszcze czeka).
**Outcome**: Utworzono wersję v0.5.0 (commit bf8ad85): historia wypowiedzi w podmenu z kopiowaniem i czyszczeniem (plik history.json), skracanie długich pauz przed transkrypcją, limit nagrania 10 min z auto-Stop i powiadomieniem. Scenariusze w docs/test-scenarios/v0.5.0.md (pokrywają zadanie 7.7), migawka Specky 01M4K5RCDQ42FW1W9DW3S41FHW. Wersja lokalnie do czasu git push origin v0.5.0; v0.4.0 również nadal niewypchnięta.
**Impact**: tag v0.5.0, Specky, zadanie 7.7

## 2026-10-10 17:17 — Zarządzanie modelem: co już jest
**Kind**: question
**Question/context**: Właściciel chciałby widzieć, jaki model jest wybrany, wgrywać inne z listy, usuwać je i czyścić historię; prosi o sprawdzenie, co już istnieje.
**Outcome**: Jest: jeden model large-v3-turbo w stałej lokalizacji, pobieranie z postępem w menu, „Ponów pobieranie”, czyszczenie historii (7.6). Nie ma: informacji o modelu w menu, listy modeli, przełączania ani usuwania (tylko ręcznie z katalogu). Wybór innego modelu koliduje z VA-STT-1 i wytyczną techniczną (narzucony large-v3-turbo) — wymaga decyzji właściciela w Specky. Zaproponowano dwa warianty: (a) podmenu „Model” informacyjne z usuwaniem/ponownym pobraniem, (b) katalog modeli GGML z przełączaniem (osobna faza, zmiana VA-STT-1). Czeka na wybór właściciela.
**Impact**: przyszłe wymaganie VA-MODEL-2; VA-STT-1

## 2026-10-10 17:17 — Ustawienie model_path ignorowane przez aplikację
**Kind**: bug
**Bug source**: task 5.1 (startup.rs sprawdza model tylko w stałym katalogu; pole z 1.3)
**Detected by**: telesma
**Question/context**: Przy przeglądzie obsługi modelu okazało się, że pole `model_path` z config.toml (opisane w README) honoruje tylko `va-dev transcribe`, a aplikacja paska menu zawsze używa ~/Library/Application Support/VoiceAsystent/models/ggml-large-v3-turbo.bin.
**Outcome**: Zgłoszone właścicielowi; do naprawy razem z zarządzaniem modelem (albo honorować pole w startup.rs, albo usunąć je z konfiguracji i README). Bez zmian w kodzie do decyzji.
**Impact**: apps/voice-asystent/src/startup.rs, README (sekcja Konfiguracja)

## 2026-10-10 17:23 — Propozycje VA-MODEL-2 i VA-MODEL-3 w Specky
**Kind**: feature
**Question/context**: Właściciel przyjął rekomendację (wariant informacyjny podmenu „Model” bez przełączania modeli) i polecił działać.
**Outcome**: Wystawiono wsad 01M4K6JNNT5ZPGGVCXY8D7AMAP: VA-MODEL-2 (podmenu „Model”: nazwa, rozmiar, stan; „Pokaż w Finderze”, „Usuń model” z potwierdzeniem, „Pobierz ponownie”; 5 kryteriów) i VA-MODEL-3 (aplikacja honoruje model_path z config.toml — naprawa błędu z 5.1; 4 kryteria). Czeka na akceptację właściciela; po niej następny start Ralpha założy Fazę 8. VA-STT-1 bez zmian.
**Impact**: Specky; przyszła Faza 8 (startup.rs, tray_menu.rs, download.rs, va-model)

## 2026-10-10 17:24 — Akceptacja VA-MODEL-2 i VA-MODEL-3
**Kind**: feature
**Question/context**: Właściciel zaakceptował wsad 01M4K6JNNT5ZPGGVCXY8D7AMAP.
**Outcome**: VA-MODEL-2 (01M4K6M2ZC8PQ1AE9FQNFWQ1DA) i VA-MODEL-3 (01M4K6M32Y0MQ3JPWM4XJ4CRAZ) zaakceptowane na wyraźną prośbę właściciela. Przegląd sprzeczności: VA-MODEL-3 uzupełnia VA-MODEL-1 (bez model_path zachowanie bez zmian), VA-MODEL-2 nie zmienia VA-STT-1. Mapa w telesma/state/SPECKY.md uzupełniona; następny start Ralpha w trybie 1 założy pliki w telesma/changes/ i Fazę 8.
**Impact**: Specky; Faza 8

## 2026-10-10 18:16 — Wydanie wersji v0.6.0
**Kind**: operations
**Question/context**: Właściciel przyjął propozycję wydania po zielonym pełnym suicie po zadaniach 8.1–8.4 (testy ręczne 7.7 i 8.5 jeszcze czekają).
**Outcome**: Utworzono wersję v0.6.0 (commit 91c33bd): podmenu „Model” (stan, Pokaż w Finderze, Usuń z potwierdzeniem, Pobierz ponownie bez restartu) i naprawa model_path. Scenariusze w docs/test-scenarios/v0.6.0.md (pokrywają zadanie 8.5), migawka Specky 01M4K9P49FEQ7NR1ND94TC8NWZ. Wersja lokalnie do czasu git push origin v0.6.0; v0.4.0 i v0.5.0 również niewypchnięte.
**Impact**: tag v0.6.0, Specky, zadanie 8.5

## 2026-10-10 18:28 — Co jeszcze dodać lub poprawić (przegląd)
**Kind**: question
**Question/context**: Właściciel zapytał, co według Ralpha warto jeszcze dodać albo poprawić.
**Outcome**: Zaproponowano listę z priorytetami: (1) brak sygnału „transkrypcja gotowa” — powiadomienie/dźwięk po zapisie do schowka; (2) start aplikacji opóźniony liczeniem SHA-256 modelu 1,6 GB przy każdym uruchomieniu — pamiętać sumę po rozmiarze i dacie pliku; (3) ustawienia w menu zamiast ręcznej edycji config.toml (język, limit); (4) uruchamianie przy logowaniu; (5) wariant skwantyzowany large-v3-turbo q5_0 (~0,6 GB) jako opcja — nadal ten sam model; (6) zabezpieczenia Whispera przed halucynacjami (próg no-speech, język pl zamiast auto); (7) retencja logów; (8) CI budujące .dmg przy tagu; (9) notaryzacja Apple — wymaga konta deweloperskiego; (10) „Pokaż logi” w menu. Odradzono: sprawdzanie aktualizacji (koliduje z zasadą zero ruchu sieciowego po pobraniu modelu) i automatyczne wklejanie (wymaga Dostępności). Bez zmian w planie — czeka na wybór właściciela.
**Impact**: brak (propozycje)

## 2026-10-10 19:03 — Propozycje Fazy 9 w Specky (8 wymagań)
**Kind**: feature
**Question/context**: Właściciel wybrał z listy ulepszeń wszystko oprócz notaryzacji Apple (9) i runnera z GPU w CI (10).
**Outcome**: Wystawiono wsad 01M4KC9EX5DBY9R1XYWNVHGX4Z: VA-UX-1 (sygnał „transkrypcja gotowa”: powiadomienie z początkiem tekstu, opcjonalny dźwięk, wyłączalne), VA-PERF-1 (szybki start: suma SHA-256 pamiętana po rozmiarze i dacie pliku), VA-SET-1 (podmenu Ustawienia: język, limit nagrania, bez restartu), VA-SET-2 (uruchamianie przy logowaniu przez LaunchAgent), VA-OPS-1 (Pokaż logi, retencja 7 dni), VA-STT-3 (parametry Whispera przeciw halucynacjom, szum → pusty wynik), VA-MODEL-4 (wariant q5_0 do wyboru w podmenu Model), VA-CI-1 (tag v* → .dmg w wydaniu GitHub). Czeka na akceptację właściciela; po niej następny start Ralpha założy Fazę 9.
**Impact**: Specky; przyszła Faza 9

## 2026-10-10 19:16 — Akceptacja ośmiu wymagań Fazy 9
**Kind**: feature
**Question/context**: Właściciel zaakceptował wsad 01M4KC9EX5DBY9R1XYWNVHGX4Z.
**Outcome**: VA-UX-1, VA-PERF-1, VA-SET-1, VA-SET-2, VA-OPS-1, VA-STT-3, VA-MODEL-4, VA-CI-1 zaakceptowane na wyraźną prośbę właściciela. Przegląd sprzeczności: brak; jedna uwaga — pamiętanie sumy SHA-256 (VA-PERF-1) nie wykryje uszkodzenia pliku bez zmiany rozmiaru i daty, co jest świadomym kompromisem (uszkodzenie przy zapisie zmienia datę). Mapa w telesma/state/SPECKY.md uzupełniona; następny start Ralpha w trybie 1 założy pliki w telesma/changes/ i Fazę 9.
**Impact**: Specky; Faza 9

## 2026-10-10 21:45 — Wydanie wersji v0.7.0
**Kind**: operations
**Question/context**: Właściciel przyjął propozycję wydania („Tak”) po zielonym pełnym suicie po zadaniach 9.1–9.11 (testy ręczne 7.7, 8.5 i 9.10 jeszcze czekają).
**Outcome**: Utworzono wersję v0.7.0 (tag z opisem na commicie 2dbd1db). Zakres: sygnał „transkrypcja gotowa”, szybki start, Ustawienia (język, limit, autostart), Pokaż logi i retencja, parametry Whispera, wariant q5_0, wydanie z CI. Scenariusze w docs/test-scenarios/v0.7.0.md (pokrywają zadanie 9.10), migawka Specky 01M4KNBCV4MJQ3SRSPZDH1MGGK. Tag tylko lokalnie do czasu git push origin v0.7.0; na GitHubie nie ma żadnego tagu v*, więc workflow wydania jeszcze się nie uruchomił.
**Impact**: tag v0.7.0, Specky, zadanie 9.10

## 2026-10-11 11:05 — Asystent mobilny: nowe repozytorium czy ten projekt
**Kind**: question
**Question/context**: Właściciel rozważa asystenta na Androida, który nasłuchuje, rozumie treść i odpowiada głosem w sposób jak najbliższy rozmowie z człowiekiem; pyta, czy dodać go do tego projektu, czy założyć nowe repozytorium.
**Outcome**: Rekomendacja: nowe repozytorium i nowy projekt w Specky. Obecny projekt ma przypięte ograniczenia sprzeczne z nowym produktem (tylko macOS, tylko GPU Metal, zakres bez LLM i TTS), a wspólnego kodu jest niewiele: tylko algorytmy ciszy z va-audio, które łatwo skopiować albo wydzielić. Mobilny asystent rozmowy to inny produkt (pętla: wykrywanie mowy → STT → model językowy → TTS, przerywanie w pół zdania, opóźnienie poniżej ~1 s). Przed planem trzeba ustalić: model w chmurze czy na telefonie, stack aplikacji Android i prywatność nagrań.
**Impact**: brak zmian w tym projekcie; decyzja o repozytorium czeka na właściciela

## 2026-10-12 — Jak założyć nowy projekt (asystent Android)
**Kind**: question
**Question/context**: Właściciel zdecydował, że asystent mobilny będzie osobnym projektem, i zapytał, jak go założyć.
**Outcome**: Ustaliliśmy, że asystent na Androida powstanie w osobnym repozytorium. Kroki: nowy katalog i repo na GitHubie, `ralph` w nowym katalogu (szkielet telesma/spec/ i telesma/), opis produktu w telesma/spec/, doprecyzowanie `ralph -i`, nowy projekt w Specky i podpięcie przez `ralph --integracje`. W telesma/guidelines/ brakuje wytycznych dla Kotlina/Androida — trzeba je dodać, zanim Ralph zacznie pisać kod (inaczej zadania z takim tagiem zablokują pętlę).
**Impact**: brak zmian w tym projekcie

## 2026-10-12 — Wydanie v0.7.0 z CI i publiczne repo bez Ralpha
**Kind**: question
**Question/context**: Właściciel wypchnął wersję v0.7.0 i szukał .dmg w dist/; potem zapytał, jak udostępniać repozytorium publicznie bez plików Ralpha (na razie pomysły do przekazania do projektu Ralph).
**Outcome**: Workflow wydania przeszedł (testy, obraz, kontrola, wydanie GitHub z VoiceAsystent-0.7.0.dmg, 4,0 MB); .dmg jest w wydaniu na GitHubie, a dist/ zapełnia tylko lokalny build-dmg.sh. Rekomendowany kierunek: prywatne repo robocze z Ralphem + publiczne lustro generowane automatycznie (filtr ścieżek i komunikatów commitów) albo stan Ralpha poza repozytorium kodu. Historia publicznego repo już zawiera pliki Ralpha — samo usunięcie ich jej nie czyści.
**Impact**: brak zmian w kodzie; pomysły dla frameworka Ralph

## 2026-10-13 — Brak sygnału „można wklejać” i żółte kółko w trakcie transkrypcji
**Kind**: bug
**Bug source**: task 9.3
**Detected by**: scenario
**Question/context**: Właściciel przeszedł testy ręczne (7.7, 8.5, 9.10) — wszystko działa poza tym, że po zatrzymaniu nagrania nie dostaje komunikatu, że może wklejać; proponuje żółte kółko na czas transkrypcji.
**Outcome**: Powiadomienie „Transkrypcja w schowku” (VA-UX-1) jest wysyłane bez błędu w logach, ale biblioteka powiadomień podpisuje je jako Finder (aplikacja nie ustawia własnego identyfikatora), więc macOS może je chować — do potwierdzenia u właściciela, potem poprawka jako błąd 9.3. Żółte kółko to zmiana VA-UI-1: propozycja w Specky (wsad 01M4TF94V3G6Y996N62Q3N1M02: nowa treść, kryterium „żółte przez całą transkrypcję”, „po zakończeniu szare, także przy ciszy i błędzie”, „w trakcie nie da się zacząć nagrania”) czeka na akceptację właściciela.
**Impact**: VA-UI-1, VA-UX-1, zadania 9.3, 9.10; nowa faza po akceptacji

## 2026-10-13 — Powiadomienia: blokada w ustawieniach macOS, nie błąd
**Kind**: question
**Question/context**: Właściciel ustalił, że powiadomienia były zablokowane w ustawieniach macOS; po odblokowaniu widać je przy zegarku.
**Outcome**: Zgłoszenie błędu powiadomień wycofane (plik usunięty z telesma/changes/ przed przetworzeniem), testy 7.7 i 9.10 zaliczone w całości. Zostaje do rozważenia drobna poprawka: powiadomienia są podpisywane jako Finder zamiast VoiceAsystent. Żółte kółko (VA-UI-1) zatwierdzone w Specky 2026-10-13 — wejdzie do planu przy następnej synchronizacji.
**Impact**: telesma/changes/, VA-UI-1

## 2026-10-13 — Powiadomienia mają nazwę VoiceAsystent
**Kind**: question
**Question/context**: Właściciel pokazał zrzut Ustawień → Powiadomienia z osobną pozycją „VoiceAsystent”.
**Outcome**: Moje wcześniejsze przypuszczenie, że powiadomienia są podpisywane jako Finder, było błędne (wyciągnięte z kodu biblioteki, bez sprawdzenia w systemie). Propozycja wpisu do BACKLOG odpada; notatka w REPORT.md poprawiona.
**Impact**: brak

## 2026-10-13 — Uruchomienie Ralpha w trybie 1 zablokowane na uprawnieniach
**Kind**: operations
**Question/context**: Właściciel poprosił o uruchomienie Ralpha w trybie 1; sesja w tle zsynchronizowała Specky (VA-UI-1 v2 → Faza 5.Z1, PR #66), ale stanęła na TELESMA BLOCKED, bo bez sędziego uprawnień każde `cargo` i `gh pr merge` wymaga zgody człowieka.
**Outcome**: PR #66 zmergowany ręcznie z tej sesji (checki zielone, Merge: agent). Do decyzji właściciela: dopisać komendy cargo (test, clippy, fmt, check, build) do allowlisty w telesma/config.md albo uruchamiać tryb 1 interaktywnie (`ralph -m 1 -i`).
**Impact**: telesma/config.md (Dozwolone komendy), Faza 5.Z1

## 2026-10-13 14:18 — Wersja v0.8.0 (żółte kółko)
**Kind**: operations
**Question/context**: Właściciel zgodził się na propozycję wydania v0.8.0 po zmergowaniu 5.Z1.1.
**Outcome**: Utworzono wersję v0.8.0 (51c6e4f): żółte kółko przez cały czas transkrypcji, szare po zapisie do schowka, brak nowego nagrania przy żółtym kółku. Pełny zestaw na main 217/0, scenariusze w docs/test-scenarios/v0.8.0.md (PR #69), migawka Specky 01M4TK6HTA9257GJR6WPTRW1F0. Wersja jest tylko lokalnie — workflow wydania ruszy po `git push origin v0.8.0`; test ręczny ⛔ 5.Z1.2 pokrywa się ze scenariuszami 1.1–2.2.
**Impact**: Faza 5.Z1, wersja v0.8.0

## 2026-11-04 08:31 — Tryb nagrywania wideokonferencji (Meet/Teams)
**Kind**: feature
**Question/context**: Właściciel chce włączać nagrywanie VoiceAsystenta w trakcie rozmowy w Google Meet/Teams (jego głos słyszą rozmówcy, a asystent nagrywa równolegle) i po zakończeniu dostać zwykłą transkrypcję; pyta, czy to możliwe i jak to zrobić.
**Outcome**: No decision — awaits: wybór wariantu przez właściciela. Wyjaśniono, że własny głos da się nagrywać już dziś bez zmian w kodzie (macOS pozwala kilku aplikacjom czytać ten sam mikrofon; limit do 30 min z menu, dłuższy przez config.toml; bufor trzymany w RAM w częstotliwości urządzenia), a obie strony rozmowy — już dziś przez BlackHole + urządzenie zagregowane w Konfiguratorze MIDI Audio, bez zmian w kodzie. Pełny „Tryb spotkania” w aplikacji wymaga przechwytywania dźwięku systemowego (Core Audio process tap / ScreenCaptureKit, uprawnienie „Nagrywanie ekranu i dźwięku systemowego”, najpewniej mały pomocnik w Swifcie, bo Rust ma `unsafe_code = deny`), zapisu długiego nagrania na dysk zamiast RAM, osobnej pozycji w menu/skrótu i opcjonalnie podziału „Ja:/Rozmówca:” z dwóch ścieżek po znacznikach czasu. Rekomendacja: najpierw sprawdzić wariant z BlackHole, potem zgłosić wymagania VA-CONF-* w Specky.
**Impact**: potencjalna nowa faza (nagrywanie spotkań); wymagania do zaproponowania w Specky po decyzji

## 2026-11-04 08:43 — Tryb spotkania: poziom 3, propozycje VA-CONF-1..4
**Kind**: feature
**Question/context**: Właściciel wybrał od razu pełny tryb spotkania w aplikacji (poziom 3) bez testowania wariantu z BlackHole, który uważa za zawodny.
**Outcome**: Ustaliliśmy: wbudowany „Tryb spotkania” z przechwytywaniem dźwięku systemowego przez Core Audio process tap (macOS 14.2+, uprawnienie „Nagrywanie dźwięku systemowego”), bez BlackHole i bez zmiany wyjścia audio. Wysłano do Specky wsad 01M6JR5MYW2TRS1V5MP4N0CQCG w nowej sekcji „Nagrywanie spotkań”: VA-CONF-1 (pozycja menu i ctrl+cmd+m), VA-CONF-2 (process tap, zgoda, odmowa), VA-CONF-3 (limit 120 min, zapis na dysk zamiast RAM), VA-CONF-4 (dwie ścieżki, etykiety „Ja:”/„Rozmówcy:”). No decision — awaits: akceptacja wsadu w Specky oraz wybór sposobu wywołania API Core Audio (izolowany moduł z unsafe vs pomocnik w Swifcie), bo projekt ma unsafe_code = deny i VA-TECH-1 (Rust).
**Impact**: nowa faza po akceptacji w Specky (synchronizacja 18.1 → telesma/changes/)

## 2026-11-04 08:44 — Akceptacja VA-CONF-1..4 w Specky
**Kind**: feature
**Question/context**: Właściciel polecił zaakceptować w Specky cały wsad wymagań trybu spotkania.
**Outcome**: Wsad 01M6JR5MYW2TRS1V5MP4N0CQCG zaakceptowany na wyraźną prośbę właściciela: VA-CONF-1..4 są wymaganiami (sekcja „Nagrywanie spotkań”). Przegląd spójności czterech wymagań z resztą specyfikacji nie wykazał sprzeczności. No decision — awaits: wybór sposobu wywołania Core Audio (moduł z unsafe vs pomocnik w Swifcie) przed zaplanowaniem fazy.
**Impact**: następna sesja autonomiczna: synchronizacja Specky → telesma/changes/ → nowa faza

## 2026-11-04 08:48 — Core Audio process tap w Ruście (izolowane unsafe)
**Kind**: feature
**Question/context**: Właściciel wybrał rekomendowany sposób wywołania API Core Audio dla trybu spotkania: Rust zamiast pomocnika w Swifcie.
**Outcome**: Ustaliliśmy: przechwytywanie dźwięku systemowego powstaje w nowym crate'cie `va-system-audio` (crates/system-audio) na objc2-core-audio 0.3 (CATapDescription, AudioHardwareCreateProcessTap, urządzenie zagregowane z tapem). To jedyne miejsce z `#![allow(unsafe_code)]`; każde wywołanie `unsafe` ma komentarz SAFETY, a reszta workspace zostaje przy `unsafe_code = deny`. Całość dalej w Ruście i cargo (VA-TECH-1). Pomocnik w Swifcie odrzucony: drugi język i osobne budowanie.
**Impact**: nowa faza 10 (VA-CONF-1..4)

## 2026-11-04 09:48 — Realizacja Fazy 10: tryb nagrywania spotkania
**Kind**: feature
**Question/context**: Właściciel polecił zrealizować tryb spotkania w Ruście według rekomendacji (izolowane unsafe zamiast pomocnika w Swifcie).
**Outcome**: Faza 10 zaplanowana (PR #70) i zrealizowana zadaniami 10.1–10.7 (PR #71–#77): konfiguracja, znaczniki czasu segmentów, scalanie ścieżek z etykietami, plik tymczasowy, crate va-system-audio z Core Audio process tap, kontroler i aplikacja (menu, ctrl+cmd+m, zgoda, Info.plist). Pełny zestaw 260/0, w tym prawdziwy tap i cały tor z Whisperem; VA-CONF-1..4 `code_ready`. W trakcie znaleziono i naprawiono błąd częstotliwości przechwytywania (zegar urządzenia zagregowanego, 16 kHz przy słuchawkach Bluetooth). Zostaje test ręczny ⛔ 10.8.
**Impact**: Faza 10; crates/system-audio, crates/core (meeting, meeting_capture, spool), apps/voice-asystent

## 2026-11-04 09:57 — Okno „Gdzie jest use_default?” po spotkaniu
**Kind**: bug
**Bug source**: task 9.3
**Detected by**: scenario
**Question/context**: W teście ręcznym trybu spotkania po zakończeniu nagrania macOS pokazał okno wyboru aplikacji „Gdzie jest use_default?”.
**Outcome**: Przyczyna w bibliotece mac-notification-sys: bez ustawionego nadawcy przy pierwszym powiadomieniu w procesie uruchamia AppleScript `id of application "use_default"`, a macOS nie zna takiej aplikacji. Poprawka: przy starcie aplikacja ustawia nadawcę powiadomień (io.github.mario12358.voiceasystent z bundla, Terminal przy cargo run), więc AppleScript nie rusza; test pilnuje zgodności identyfikatora z Info.plist. Według kodu okno blokuje tylko wątek powiadomienia — tekst jest już wtedy w schowku (do potwierdzenia u właściciela).
**Impact**: apps/voice-asystent/src/messages.rs, main.rs

## 2026-11-04 10:09 — Zła kolejność wypowiedzi w transkrypcji spotkania
**Kind**: bug
**Bug source**: task 10.3
**Detected by**: scenario
**Question/context**: W testowej rozmowie w Meet wypowiedzi „Ja:”/„Rozmówcy:” były w innej kolejności niż w rzeczywistości; właściciel pyta dlaczego i czy da się to przetestować automatycznie.
**Outcome**: No decision — awaits: zgoda właściciela na poprawkę. Odtworzone na sztucznej rozmowie (6 naprzemiennych wypowiedzi, `say` Zosia, prawdziwy model): Whisper skleja kilka wypowiedzi jednej osoby w jeden segment (rozmówcy: jeden segment 2,9–15,5 s z trzema wypowiedziami), a początki segmentów są nieprecyzyjne (moja druga wypowiedź od 5,9 s dostała 2,1 s), więc scalanie po początku segmentu przestawia wypowiedzi. Rekomendacja: wyznaczać tury mówców z detekcji mowy na obu ścieżkach przed transkrypcją (dokładne czasy z sygnału), transkrybować każdą turę osobno, plus test na sztucznej rozmowie jako fixture.
**Impact**: crates/core/src/meeting.rs (merge_tracks/transcribe_track), VA-CONF-4

## 2026-11-04 10:32 — Poprawka kolejności wypowiedzi w spotkaniu
**Kind**: bug
**Bug source**: task 10.3
**Detected by**: scenario
**Question/context**: Właściciel zgodził się na poprawkę kolejności wypowiedzi w transkrypcji spotkania.
**Outcome**: Ustaliliśmy i wprowadziliśmy: tury mówców wyznaczane z detekcji mowy na obu ścieżkach (czas z sygnału, przerwa < 0,4 s nie dzieli odcinka), kolejne odcinki tego samego mówcy łączone w turę, każda tura transkrybowana osobno, wynik w kolejności tur. Znaczniki segmentów Whispera nie decydują już o kolejności; mapa czasu po skróceniu pauz (10.2) usunięta. Sztuczna rozmowa z 6 naprzemiennymi wypowiedziami (tests/fixtures/dialogue_*.wav) jako test z prawdziwym modelem — przechodzi; pełny zestaw 263/0. Koszt: jedno wywołanie modelu na turę.
**Impact**: crates/core/src/meeting.rs, crates/audio/src/silence.rs (speech_regions), VA-CONF-4
