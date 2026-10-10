# DECISIONS.md — Changelog ustaleń z trybu interaktywnego

<!--
Plik tworzony i uzupełniany automatycznie przez Claude w trybie interaktywnym
(po RALPH COMPLETE / RALPH BLOCKED, gdy user prowadzi rozmowę manualnie).

Każda wymiana user↔Claude w tym trybie = jeden wpis poniżej.
Format wpisu (sekcja 10 w templates/RALPH_INSTRUCTIONS.md):

## YYYY-MM-DD HH:MM — [krótki temat]
**Rodzaj**: błąd | korekta | rozwój | pytanie | operacje
**Źródło błędu**: [tylko przy błędzie: zadanie X.Y, które to zbudowało | nieznane]
**Wykryte przez**: [tylko przy błędzie: test | scenariusz | review | ralph | właściciel | produkcja]
**Pytanie/kontekst**: [parafraza co user pytał, 1-2 zdania]
**Ustalenie**: [esencja decyzji/wyjaśnienia, 2-4 zdania, bez kodu]
**Wpływ** (opcjonalnie): [zadanie X.Y / plik / faza]

Plik jest WRITE-ONLY — Ralph go nie czyta w żadnym innym procesie.
To changelog dla człowieka, żeby pamiętać "co i dlaczego ustaliliśmy w trakcie".
-->


## 2026-10-08 22:28 — Tylko macOS, Specky źródłem wymagań
**Rodzaj**: korekta
**Pytanie/kontekst**: Właściciel odpowiedział na RALPH BLOCKED. Aplikacja ma działać wyłącznie na macOS i być instalowana z pliku instalacyjnego. Specky ma być jedynym źródłem wymagań. Repozytorium git z origin już istnieje.
**Ustalenie**: Plan przepisany od zera pod aplikację paska menu macOS: 6 faz, 45 pozycji, GPU Metal, schowek zamiast wpisywania tekstu, bez LLM i TTS, instalator .dmg. 11 wymagań (wymagania.md + WYTYCZNE_TECHNICZNE.md) zaproponowano w Specky jako wsad 01M4EK43A6GBPR63H0FV0VBCV0 do akceptacji właściciela. W wymaganiu 3 („ctrl + v”) przyjęto cmd + v ze wstępu wymagania.md.
**Wpływ**: ralph/PLAN.md (cały), Specky (11 propozycji)

## 2026-10-08 22:36 — Akceptacja wymagań, model pobierany po instalacji
**Rodzaj**: rozwój
**Pytanie/kontekst**: Właściciel zatwierdził wsad wymagań w Specky, zainstalował Rust i zdecydował, że model ma być pobierany po instalacji aplikacji.
**Ustalenie**: 11 wymagań zaakceptowanych w Specky i podpiętych do planu jako kotwice. Model nie trafia do .dmg: aplikacja pobiera go przy pierwszym uruchomieniu (z postępem, wznawianiem i sumą SHA-256). Dodano zadanie 5.6 i propozycję wymagania VA-MODEL-1 do akceptacji.
**Wpływ**: zadania 3.2, 5.6, 6.1, 6.2, 6.3

## 2026-10-08 22:43 — Bez wydania po Fazie 1, stan Specky
**Rodzaj**: pytanie
**Pytanie/kontekst**: Właściciel odrzucił wydanie v0.1.0 po Fazie 1, kazał kontynuować i zapytał, czy w Specky coś czeka.
**Ustalenie**: Brak wersji, praca idzie dalej od zadania 2.1. W Specky czeka na decyzję tylko propozycja VA-MODEL-1. Kolejka poprawek jest pusta; 3 wymagania są w realizacji, 8 otwartych. Kryteria nie dostaną dowodów z testów, dopóki nie ma CI (integracja z repozytorium w config: brak).
**Wpływ**: Faza 2

## 2026-10-08 22:58 — Pauza po zadaniu 2.2
**Rodzaj**: operacje
**Pytanie/kontekst**: W trakcie pracy właściciel poprosił o zatrzymanie po bieżącym zadaniu z planu.
**Ustalenie**: Zadanie 2.2 zostało dokończone i zacommitowane, potem praca stanęła. Następne do zrobienia jest 2.3 (przycinanie ciszy).
**Wpływ**: Faza 2

## 2026-10-08 23:10 — code_ready dla VA-PLAT-1 i VA-TECH-1
**Rodzaj**: operacje
**Pytanie/kontekst**: Właściciel w trakcie zadania 2.3 poprosił o ustawienie w Specky statusu code_ready dla VA-PLAT-1 i VA-TECH-1.
**Ustalenie**: Ustawiono code_ready dla obu wymagań (odcisk treści v1 z get_requirement). Faza 1 (szkielet macOS-only, workspace Cargo) je realizuje; status done ustala właściciel.
**Wpływ**: Specky — VA-PLAT-1, VA-TECH-1

## 2026-10-08 23:17 — Wydanie wersji v0.1.0
**Rodzaj**: operacje
**Pytanie/kontekst**: Właściciel przyjął propozycję wydania po zielonej regresji Fazy 2.
**Ustalenie**: Utworzono wersję v0.1.0 (commit 57cf6af) obejmującą fazy 1–2: szkielet macOS, konfigurację, wybór mikrofonu, nagrywanie do 16 kHz mono i przycinanie ciszy. Scenariusze testów w docs/test-scenarios/v0.1.0.md, migawka w Specky 01M4EP1HG6SQR1GC25CTTSJ18P. Wersja jest tylko lokalnie do czasu git push origin v0.1.0.
**Wpływ**: tag v0.1.0, Specky

## 2026-10-08 23:20 — cargo niedostępny w terminalu właściciela
**Rodzaj**: pytanie
**Pytanie/kontekst**: Przy scenariuszach v0.1.0 terminal zgłaszał „command not found: cargo”.
**Ustalenie**: Rust jest zainstalowany, a ~/.zshenv ładuje ~/.cargo/env; terminal był otwarty przed instalacją. Wystarczy nowy terminal albo `source "$HOME/.cargo/env"`. Hook kontroli to osobna sprawa (nie czyta ~/.zshenv) — do sprawdzenia.
**Wpływ**: docs/test-scenarios/v0.1.0.md (sekcja 0)

## 2026-10-08 23:24 — Mikrofon z konfiguracji nie ustawia się
**Rodzaj**: pytanie
**Pytanie/kontekst**: Przy scenariuszu 1.2 v0.1.0 wybór mikrofonu w config.toml nie działał.
**Ustalenie**: Plik ~/Library/Application Support/VoiceAsystent/config.toml (ani katalog) nie istnieje, więc działają wartości domyślne — kod zachowuje się poprawnie. Podano dokładne nazwy urządzeń (PXC 550, BlackHole 2ch, Mikrofon (MacBook Air)) i sposób utworzenia pliku; klucz `microphone` musi stać przed sekcją [silence].
**Wpływ**: docs/test-scenarios/v0.1.0.md (scenariusz 1.2)

## 2026-10-08 23:26 — Ustawienie mikrofonu PXC 550
**Rodzaj**: operacje
**Pytanie/kontekst**: Właściciel wybrał PXC 550 jako mikrofon do testów v0.1.0.
**Ustalenie**: Utworzono ~/Library/Application Support/VoiceAsystent/config.toml z `microphone = "PXC 550"`; `va-dev devices` pokazuje `>` przy PXC 550, pozostałe ustawienia domyślne.
**Wpływ**: scenariusz 1.2 v0.1.0

## 2026-10-08 23:44 — Wymagania czekające na akceptację w Specky
**Rodzaj**: pytanie
**Pytanie/kontekst**: Właściciel zapytał, czy w Specky są wymagania do zaakceptowania.
**Ustalenie**: Czeka jedna propozycja: VA-MODEL-1 (model pobierany po instalacji, wsad 01M4EKHX2XBS6ZSPSE9DAV9QFD, zmiana 01M4EKHXE3M2312B7EECQ55EHJ). Od niej zależą 3.2 i 5.6. Brak otwartych pytań i kolizji. Akceptacja na wyraźną prośbę właściciela.
**Wpływ**: zadania 3.2, 5.6

## 2026-10-08 23:47 — Akceptacja VA-MODEL-1
**Rodzaj**: rozwój
**Pytanie/kontekst**: Właściciel poprosił o zaakceptowanie propozycji VA-MODEL-1 w Specky.
**Ustalenie**: Zaakceptowano (wymaganie 01M4EQH9D30871ZG5JMWK8VHFV v1, 6 kryteriów: .dmg bez modelu, pobieranie z postępem przy pierwszym starcie, SHA-256 z usunięciem uszkodzonego pliku, wznawianie, komunikat zamiast nagrania przed pobraniem, zero ruchu sieciowego po pobraniu). Kotwice dopisane do 3.2, 5.6, 6.1.
**Wpływ**: zadania 3.2, 5.6, 6.1, 6.2; ralph/SPECKY.md

## 2026-10-08 23:52 — Kryteria Specky bez dowodów z CI (Rust)
**Rodzaj**: operacje
**Pytanie/kontekst**: Rozstrzygnięcie blokady: Specky nie łączył testów Rusta ze znacznikami kryteriów.
**Ustalenie**: Właściciel wyjaśnił, że Specky czyta znaczniki tylko w .py/.js/.ts. Ustaliliśmy wariant (c): pracujemy dalej bez dowodów z CI, znaczniki `specky: crit` w testach zostają na przyszłość, kryteria właściciel odhacza ręcznie w komentarzu Specky na PR. Blokada usunięta.
**Wpływ**: sekcja 18.5 kontraktu CI; ralph/BLOCKED.md usunięty; PROJECT_CONTEXT (📌)

## 2026-10-09 00:40 — Naprawa kontroli zaleznosci i PATH hooka
**Rodzaj**: błąd
**Źródło błędu**: nieznane
**Wykryte przez**: ralph
**Pytanie/kontekst**: Właściciel naprawił kontrolę `zaleznosci` (wywracała się na binarnych WAV) i PATH hooka (brak ~/.cargo/bin), prosząc o dołączenie poprawek do PR #9 i merge.
**Ustalenie**: Poprawki dołączone do gałęzi PR #9; amend ostatniego commita nie wystarczył (dowód dla wcześniejszego commita wciąż „nie wykonała się”), więc oba commity gałęzi scalone `reset --soft` i zacommitowane ponownie przez hook — wszystkie 11 kontroli przeszło, w tym clippy/fmt. PR #9 zmergowany, blokada usunięta.
**Wpływ**: zadanie 3.3, ralph-kontrole/zaleznosci/run.py, ralph-kontrole/ralph-kontrole.py

## 2026-10-09 00:55 — Wydanie wersji v0.2.0
**Rodzaj**: operacje
**Pytanie/kontekst**: Właściciel przyjął propozycję wydania po zielonej regresji Fazy 3.
**Ustalenie**: Utworzono wersję v0.2.0 (commit e338ac8): sprawdzanie GPU Metal bez fallbacku CPU, pobieranie modelu z wznawianiem i SHA-256, transkrypcja WAV na GPU (`va-dev transcribe`). Scenariusze w docs/test-scenarios/v0.2.0.md, migawka Specky 01M4EVJ3FKXBKZG942NG0KYFVM. Wersja lokalnie do czasu git push origin v0.2.0.
**Wpływ**: tag v0.2.0, Specky

## 2026-10-09 01:00 — Pauza po zadaniu 4.1
**Rodzaj**: operacje
**Pytanie/kontekst**: W trakcie finalizacji 4.1 właściciel poprosił o dokończenie zadania i zatrzymanie pracy.
**Ustalenie**: Zadanie 4.1 (schowek) zmergowane (PR #13) i odhaczone, praca zatrzymana. Następne do zrobienia jest 4.2 (automat stanów nagrywania).
**Wpływ**: Faza 4

## 2026-10-09 08:30 — Bez wydania po Fazie 4
**Rodzaj**: operacje
**Pytanie/kontekst**: Propozycja wydania v0.3.0 po zielonej regresji Fazy 4 (zmiany bez widocznego efektu dla użytkownika).
**Ustalenie**: Właściciel odrzucił wydanie zgodnie z rekomendacją — następna propozycja po Fazie 5 (ikona, skróty, schowek w działającej aplikacji). Praca idzie dalej od 5.1.
**Wpływ**: Faza 5

## 2026-10-09 09:52 — Wydanie wersji v0.3.0
**Rodzaj**: operacje
**Pytanie/kontekst**: Właściciel przyjął propozycję wydania po zielonej regresji Fazy 5.
**Ustalenie**: Utworzono wersję v0.3.0 (commit 23ec3de) obejmującą fazy 4–5: działający asystent w pasku menu (kliknięcie i skróty, schowek, menu mikrofonu, komunikaty, pobieranie modelu w tle). Scenariusze w docs/test-scenarios/v0.3.0.md, migawka Specky 01M4FTCC8WGF4QWM98D0CE7JEG. Wersja lokalnie do czasu git push origin v0.3.0.
**Wpływ**: tag v0.3.0, Specky

## 2026-10-09 11:15 — Wydanie wersji v0.4.0
**Rodzaj**: operacje
**Pytanie/kontekst**: Właściciel przyjął propozycję wydania po zielonym pełnym suicie po zadaniach 6.1–6.3 (instalator).
**Ustalenie**: Utworzono wersję v0.4.0 (commit 1ae141a): bundle VoiceAsystent.app z własną ikoną i polskim opisem uprawnienia mikrofonu, obraz .dmg z przeciąganiem do Applications, README i RUNBOOK. Scenariusze w docs/test-scenarios/v0.4.0.md, migawka Specky 01M4FZ5ZY6SRS6JTDZKJQNVK28. Wersja lokalnie do czasu git push origin v0.4.0. Test ręczny 6.4 nadal czeka na właściciela — Faza 6 bez tagu fazy.
**Wpływ**: tag v0.4.0, Specky, zadanie 6.4

## 2026-10-09 12:33 — Stan Specky po wydaniu v0.4.0
**Rodzaj**: pytanie
**Pytanie/kontekst**: Właściciel zapytał, co nowego w Specky.
**Ustalenie**: Od kursora 709 jedyna nowa pozycja to migawka v0.4.0 (01M4FZ5ZY6SRS6JTDZKJQNVK28). Kolejka pracy pusta: 0 do poprawy, 0 do zbudowania, 8 wymagań code_ready, 4 done (jedno z ostatnio oznaczonych code_ready właściciel przeniósł na done). Brak dryfu, brak czekających propozycji, brak czerwonych PR-ów.
**Wpływ**: brak

## 2026-10-09 12:34 — Ponowne sprawdzenie nowości
**Rodzaj**: pytanie
**Pytanie/kontekst**: Właściciel zapytał, czy jest coś nowego (bez wskazania kanału).
**Ustalenie**: Sprawdzone wszystkie kanały: Specky bez nowych pozycji od kursora 710, changes/ zawiera tylko szablon PRZYKLAD.md (bez zmian do przetworzenia), brak otwartych PR-ów, main równe z origin/main. Jedyne, co czeka: wersja v0.4.0 nie jest jeszcze wypchnięta (git push origin v0.4.0) i test ręczny 6.4.
**Wpływ**: brak

## 2026-10-09 12:35 — Nowe wymaganie: wsparcie Windows
**Rodzaj**: pytanie
**Pytanie/kontekst**: Właściciel dodał przez chat Specky wymaganie „Aplikacja ma wspierać również Windows” (01M4G3NMKKEBCK15MKCVXN4789, zaakceptowane, 3 kryteria) i zapytał o nie.
**Ustalenie**: Wymaganie koliduje z VA-PLAT-1 (wyłącznie macOS, status done, kryterium „brak konfiguracji budowania dla Windows”), VA-STT-2 (tylko GPU Metal) i VA-PLAT-2 (tylko .dmg); kryterium 2 mówi o „rozmowie z LLM”, której w projekcie nie ma (decyzja 2026-10-08: bez LLM). Przedstawiono właścicielowi listę rozstrzygnięć do podjęcia w Specky (zastąpienie VA-PLAT-1, GPU na Windows: CUDA/Vulkan, skróty i schowek na Windows, LLM tak/nie, external_ref i sekcja) oraz szacunek: osobna faza, bez maszyny z Windows nieweryfikowalna. Bez zmian w planie do decyzji właściciela.
**Wpływ**: VA-PLAT-1, VA-STT-2, VA-PLAT-2; przyszła Faza 7

## 2026-10-09 22:34 — Cofnięcie wymagania o Windows w Specky
**Rodzaj**: korekta
**Pytanie/kontekst**: Właściciel polecił cofnąć w Specky obsługę Windows i Linux — projekt ma zostać wyłącznie na macOS.
**Ustalenie**: Wymaganie 01M4G3NMKKEBCK15MKCVXN4789 („wspierać również Windows”) wycofane (deprecate bez następcy): propozycja 01M4H5YZBGV79B02KW3B4T66NJ zaakceptowana na wyraźną prośbę właściciela. VA-PLAT-1 (wyłącznie macOS) obowiązuje bez zmian; osobnego wymagania o Linux nigdy nie było. Plan bez zmian — Faza 7 nie powstaje.
**Wpływ**: Specky; decyzja 📌 „wyłącznie macOS” w PROJECT_CONTEXT pozostaje

## 2026-10-10 12:07 — Historia wypowiedzi (pomysł na nową funkcję)
**Rodzaj**: pytanie
**Pytanie/kontekst**: Właściciel chce mieć „schowek na wypowiedzi” — możliwość wrócenia do wcześniejszych transkrypcji i skopiowania ich ponownie; pyta, jak do tego podejść.
**Ustalenie**: Rekomendacja: podmenu „Historia” pod prawym kliknięciem ikony (ostatnie N wpisów, skrót tekstu + godzina, kliknięcie kopiuje pełny tekst do schowka, pozycja „Wyczyść historię”); logika w va-core (bufor po zdarzeniu Delivered), zapis na dysk w katalogu danych aplikacji z limitem wpisów — wymaga decyzji właściciela, bo to pierwsze trwałe przechowywanie treści (dotąd tylko schowek). Ścieżka formalna: nowe wymaganie w Specky (np. VA-HIST-1) → changes/ → Faza 7 (ok. 4–5 zadań). Na razie bez zmian w planie.
**Wpływ**: przyszła Faza 7; decyzja o trwałości i prywatności historii

## 2026-10-10 15:24 — Historia wypowiedzi zapisywana na dysku; limit i pauzy w nagraniu
**Rodzaj**: rozwój
**Pytanie/kontekst**: Właściciel zdecydował, że historia wypowiedzi ma być zapisywana na dysku, i zapytał, jak długie mogą być nagrania oraz czy 5-minutowe nagranie z długimi przerwami w mówieniu zostanie obsłużone.
**Ustalenie**: Zaproponowano w Specky wymaganie VA-HIST-1 (podmenu „Historia”, kliknięcie kopiuje tekst, limit 30 wpisów, zapis w katalogu danych aplikacji, „Wyczyść historię”, treść poza logami) — czeka na akceptację właściciela, potem changes/ → Faza 7. Nagrania: limit 5 min (max_recording_secs=300, zmienny w config.toml); dokładnie 5 min mieści się, nadwyżka jest po cichu odrzucana (tylko ostrzeżenie w logu, ikona dalej czerwona) — luka do naprawy. Cisza jest przycinana tylko na brzegach; długie pauzy w środku idą do Whispera i grożą halucynacjami w oknach 30 s bez mowy. Zaproponowano: kompresję pauz wewnętrznych przed transkrypcją i powiadomienie/auto-stop przy limicie — do decyzji właściciela.
**Wpływ**: Specky VA-HIST-1; przyszła Faza 7; crates/audio (pauzy), recorder (limit)

## 2026-10-10 15:29 — Propozycje VA-REC-5 i VA-REC-6 w Specky
**Rodzaj**: rozwój
**Pytanie/kontekst**: Właściciel polecił wystawić przez Specky dwa wymagania zaproponowane przy analizie długich nagrań.
**Ustalenie**: Zaproponowano w tym samym wsadzie co VA-HIST-1: VA-REC-5 (skracanie pauz wewnętrznych > 1,5 s do 0,5 s przed transkrypcją, 4 kryteria) i VA-REC-6 (limit nagrania domyślnie 10 min, automatyczny Stop z transkrypcją i powiadomienie przy limicie, 4 kryteria). Wsad 01M4JZS73RFCBAXE91TSTJCG2V czeka na akceptację właściciela; po niej następny start Ralpha założy pliki w changes/ i Fazę 7.
**Wpływ**: Specky; przyszła Faza 7 (crates/audio, recorder, kontroler, powiadomienia)

## 2026-10-10 15:32 — Akceptacja VA-HIST-1, VA-REC-5, VA-REC-6; pytanie o test ręczny
**Rodzaj**: rozwój
**Pytanie/kontekst**: Właściciel potwierdził treść trzech propozycji i polecił je zaakceptować w Specky; push wersji v0.4.0 zostawia na później; zapytał, czym jest test ręczny.
**Ustalenie**: Wsad 01M4JZS73RFCBAXE91TSTJCG2V zaakceptowany na wyraźną prośbę właściciela: VA-HIST-1 (01M4K06AGAV8G79RJMXCY0SQA5), VA-REC-5 (01M4K06AQB7B8055HQ21BY4F4Z), VA-REC-6 (01M4K06AXGNSZ9XEZ3E9VFYND0). Przegląd sprzeczności z resztą projektu: brak (VA-REC-6 uzupełnia VA-REC-2, VA-REC-5 zachowuje VA-REC-3). Mapa w ralph/SPECKY.md uzupełniona; następny start Ralpha założy changes/ i Fazę 7. Test ręczny = zadanie ⛔ 6.4 z planu wg docs/test-scenarios/v0.4.0.md.
**Wpływ**: Specky; Faza 7; zadanie 6.4

## 2026-10-10 15:34 — Wynik testu ręcznego 6.4: wszystko OK
**Rodzaj**: operacje
**Pytanie/kontekst**: Właściciel zgłosił, że test ręczny 6.4 (instalacja z .dmg, nagrywanie, wklejanie, zmiana mikrofonu) wykonał 2026-10-09 i wszystko działa.
**Ustalenie**: Zadanie 6.4 odhaczone, wyniki wpisane do docs/test-scenarios/v0.4.0.md (wszystkie scenariusze [x], 2.2 nie do sprawdzenia na dev, decyzja: gotowa do wdrożenia) i do REPORT.md. Plan 47/47. Regresja Fazy 6 na main uruchomiona; po zieleni tag ralph/faza-6 i PR stanu. Push wersji v0.4.0 nadal po stronie właściciela.
**Wpływ**: Faza 6 zamknięta; tag ralph/faza-6

## 2026-10-10 17:08 — Wydanie wersji v0.5.0
**Rodzaj**: operacje
**Pytanie/kontekst**: Właściciel przyjął propozycję wydania po zielonym pełnym suicie po zadaniach 7.1–7.6 (test ręczny 7.7 jeszcze czeka).
**Ustalenie**: Utworzono wersję v0.5.0 (commit bf8ad85): historia wypowiedzi w podmenu z kopiowaniem i czyszczeniem (plik history.json), skracanie długich pauz przed transkrypcją, limit nagrania 10 min z auto-Stop i powiadomieniem. Scenariusze w docs/test-scenarios/v0.5.0.md (pokrywają zadanie 7.7), migawka Specky 01M4K5RCDQ42FW1W9DW3S41FHW. Wersja lokalnie do czasu git push origin v0.5.0; v0.4.0 również nadal niewypchnięta.
**Wpływ**: tag v0.5.0, Specky, zadanie 7.7

## 2026-10-10 17:17 — Zarządzanie modelem: co już jest
**Rodzaj**: pytanie
**Pytanie/kontekst**: Właściciel chciałby widzieć, jaki model jest wybrany, wgrywać inne z listy, usuwać je i czyścić historię; prosi o sprawdzenie, co już istnieje.
**Ustalenie**: Jest: jeden model large-v3-turbo w stałej lokalizacji, pobieranie z postępem w menu, „Ponów pobieranie”, czyszczenie historii (7.6). Nie ma: informacji o modelu w menu, listy modeli, przełączania ani usuwania (tylko ręcznie z katalogu). Wybór innego modelu koliduje z VA-STT-1 i wytyczną techniczną (narzucony large-v3-turbo) — wymaga decyzji właściciela w Specky. Zaproponowano dwa warianty: (a) podmenu „Model” informacyjne z usuwaniem/ponownym pobraniem, (b) katalog modeli GGML z przełączaniem (osobna faza, zmiana VA-STT-1). Czeka na wybór właściciela.
**Wpływ**: przyszłe wymaganie VA-MODEL-2; VA-STT-1

## 2026-10-10 17:17 — Ustawienie model_path ignorowane przez aplikację
**Rodzaj**: błąd
**Źródło błędu**: zadanie 5.1 (startup.rs sprawdza model tylko w stałym katalogu; pole z 1.3)
**Wykryte przez**: ralph
**Pytanie/kontekst**: Przy przeglądzie obsługi modelu okazało się, że pole `model_path` z config.toml (opisane w README) honoruje tylko `va-dev transcribe`, a aplikacja paska menu zawsze używa ~/Library/Application Support/VoiceAsystent/models/ggml-large-v3-turbo.bin.
**Ustalenie**: Zgłoszone właścicielowi; do naprawy razem z zarządzaniem modelem (albo honorować pole w startup.rs, albo usunąć je z konfiguracji i README). Bez zmian w kodzie do decyzji.
**Wpływ**: apps/voice-asystent/src/startup.rs, README (sekcja Konfiguracja)

## 2026-10-10 17:23 — Propozycje VA-MODEL-2 i VA-MODEL-3 w Specky
**Rodzaj**: rozwój
**Pytanie/kontekst**: Właściciel przyjął rekomendację (wariant informacyjny podmenu „Model” bez przełączania modeli) i polecił działać.
**Ustalenie**: Wystawiono wsad 01M4K6JNNT5ZPGGVCXY8D7AMAP: VA-MODEL-2 (podmenu „Model”: nazwa, rozmiar, stan; „Pokaż w Finderze”, „Usuń model” z potwierdzeniem, „Pobierz ponownie”; 5 kryteriów) i VA-MODEL-3 (aplikacja honoruje model_path z config.toml — naprawa błędu z 5.1; 4 kryteria). Czeka na akceptację właściciela; po niej następny start Ralpha założy Fazę 8. VA-STT-1 bez zmian.
**Wpływ**: Specky; przyszła Faza 8 (startup.rs, tray_menu.rs, download.rs, va-model)

## 2026-10-10 17:24 — Akceptacja VA-MODEL-2 i VA-MODEL-3
**Rodzaj**: rozwój
**Pytanie/kontekst**: Właściciel zaakceptował wsad 01M4K6JNNT5ZPGGVCXY8D7AMAP.
**Ustalenie**: VA-MODEL-2 (01M4K6M2ZC8PQ1AE9FQNFWQ1DA) i VA-MODEL-3 (01M4K6M32Y0MQ3JPWM4XJ4CRAZ) zaakceptowane na wyraźną prośbę właściciela. Przegląd sprzeczności: VA-MODEL-3 uzupełnia VA-MODEL-1 (bez model_path zachowanie bez zmian), VA-MODEL-2 nie zmienia VA-STT-1. Mapa w ralph/SPECKY.md uzupełniona; następny start Ralpha w trybie 1 założy pliki w changes/ i Fazę 8.
**Wpływ**: Specky; Faza 8

## 2026-10-10 18:16 — Wydanie wersji v0.6.0
**Rodzaj**: operacje
**Pytanie/kontekst**: Właściciel przyjął propozycję wydania po zielonym pełnym suicie po zadaniach 8.1–8.4 (testy ręczne 7.7 i 8.5 jeszcze czekają).
**Ustalenie**: Utworzono wersję v0.6.0 (commit 91c33bd): podmenu „Model” (stan, Pokaż w Finderze, Usuń z potwierdzeniem, Pobierz ponownie bez restartu) i naprawa model_path. Scenariusze w docs/test-scenarios/v0.6.0.md (pokrywają zadanie 8.5), migawka Specky 01M4K9P49FEQ7NR1ND94TC8NWZ. Wersja lokalnie do czasu git push origin v0.6.0; v0.4.0 i v0.5.0 również niewypchnięte.
**Wpływ**: tag v0.6.0, Specky, zadanie 8.5

## 2026-10-10 18:28 — Co jeszcze dodać lub poprawić (przegląd)
**Rodzaj**: pytanie
**Pytanie/kontekst**: Właściciel zapytał, co według Ralpha warto jeszcze dodać albo poprawić.
**Ustalenie**: Zaproponowano listę z priorytetami: (1) brak sygnału „transkrypcja gotowa” — powiadomienie/dźwięk po zapisie do schowka; (2) start aplikacji opóźniony liczeniem SHA-256 modelu 1,6 GB przy każdym uruchomieniu — pamiętać sumę po rozmiarze i dacie pliku; (3) ustawienia w menu zamiast ręcznej edycji config.toml (język, limit); (4) uruchamianie przy logowaniu; (5) wariant skwantyzowany large-v3-turbo q5_0 (~0,6 GB) jako opcja — nadal ten sam model; (6) zabezpieczenia Whispera przed halucynacjami (próg no-speech, język pl zamiast auto); (7) retencja logów; (8) CI budujące .dmg przy tagu; (9) notaryzacja Apple — wymaga konta deweloperskiego; (10) „Pokaż logi” w menu. Odradzono: sprawdzanie aktualizacji (koliduje z zasadą zero ruchu sieciowego po pobraniu modelu) i automatyczne wklejanie (wymaga Dostępności). Bez zmian w planie — czeka na wybór właściciela.
**Wpływ**: brak (propozycje)

## 2026-10-10 19:03 — Propozycje Fazy 9 w Specky (8 wymagań)
**Rodzaj**: rozwój
**Pytanie/kontekst**: Właściciel wybrał z listy ulepszeń wszystko oprócz notaryzacji Apple (9) i runnera z GPU w CI (10).
**Ustalenie**: Wystawiono wsad 01M4KC9EX5DBY9R1XYWNVHGX4Z: VA-UX-1 (sygnał „transkrypcja gotowa”: powiadomienie z początkiem tekstu, opcjonalny dźwięk, wyłączalne), VA-PERF-1 (szybki start: suma SHA-256 pamiętana po rozmiarze i dacie pliku), VA-SET-1 (podmenu Ustawienia: język, limit nagrania, bez restartu), VA-SET-2 (uruchamianie przy logowaniu przez LaunchAgent), VA-OPS-1 (Pokaż logi, retencja 7 dni), VA-STT-3 (parametry Whispera przeciw halucynacjom, szum → pusty wynik), VA-MODEL-4 (wariant q5_0 do wyboru w podmenu Model), VA-CI-1 (tag v* → .dmg w wydaniu GitHub). Czeka na akceptację właściciela; po niej następny start Ralpha założy Fazę 9.
**Wpływ**: Specky; przyszła Faza 9

## 2026-10-10 19:16 — Akceptacja ośmiu wymagań Fazy 9
**Rodzaj**: rozwój
**Pytanie/kontekst**: Właściciel zaakceptował wsad 01M4KC9EX5DBY9R1XYWNVHGX4Z.
**Ustalenie**: VA-UX-1, VA-PERF-1, VA-SET-1, VA-SET-2, VA-OPS-1, VA-STT-3, VA-MODEL-4, VA-CI-1 zaakceptowane na wyraźną prośbę właściciela. Przegląd sprzeczności: brak; jedna uwaga — pamiętanie sumy SHA-256 (VA-PERF-1) nie wykryje uszkodzenia pliku bez zmiany rozmiaru i daty, co jest świadomym kompromisem (uszkodzenie przy zapisie zmienia datę). Mapa w ralph/SPECKY.md uzupełniona; następny start Ralpha w trybie 1 założy pliki w changes/ i Fazę 9.
**Wpływ**: Specky; Faza 9

## 2026-10-10 21:45 — Wydanie wersji v0.7.0
**Rodzaj**: operacje
**Pytanie/kontekst**: Właściciel przyjął propozycję wydania („Tak”) po zielonym pełnym suicie po zadaniach 9.1–9.11 (testy ręczne 7.7, 8.5 i 9.10 jeszcze czekają).
**Ustalenie**: Utworzono wersję v0.7.0 (tag z opisem na commicie 2dbd1db). Zakres: sygnał „transkrypcja gotowa”, szybki start, Ustawienia (język, limit, autostart), Pokaż logi i retencja, parametry Whispera, wariant q5_0, wydanie z CI. Scenariusze w docs/test-scenarios/v0.7.0.md (pokrywają zadanie 9.10), migawka Specky 01M4KNBCV4MJQ3SRSPZDH1MGGK. Tag tylko lokalnie do czasu git push origin v0.7.0; na GitHubie nie ma żadnego tagu v*, więc workflow wydania jeszcze się nie uruchomił.
**Wpływ**: tag v0.7.0, Specky, zadanie 9.10
