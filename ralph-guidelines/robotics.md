---
tag: robotics
keywords: robot, robotyka, hal, sprzęt, hardware, symulator, symulacja, sim, czujnik, sonar, silnik, koła, gpio, i2c, raspberry, bezpieczeństwo, reguły, zachowania, scenariusz, deploy
---

# Wytyczne: Robotyka

Robot różni się od aplikacji tym, że błąd rusza fizycznym obiektem, a agent nie ma dostępu do
sprzętu. Jedynym sygnałem zwrotnym jest symulacja na fałszywym sprzęcie. Stąd wszystko poniżej.

## Sprzęt za jedną warstwą

- Cały dostęp do pinów, magistral (I2C, SPI, UART), audio i ekranu żyje w jednym pakiecie `hal/`
- Trzy pliki: `base.py` (abstrakcyjne interfejsy, jednostki w docstringach), `fake.py` (implementacja na modelu świata), `real.py` (biblioteki sprzętowe)
- Wybór implementacji przez zmienną środowiskową (np. `ROBOT_HAL=fake|real`), nigdy przez `if` rozsiane po kodzie
- Kod poza `hal/` nie importuje niczego sprzętowego (`RPi.GPIO`, `smbus`, `pyaudio`, `picamera`). Test: `grep` po importach spoza `hal/` jest pusty
- `real.py` pisze człowiek. Agent zostawia szkielet: każda metoda `raise NotImplementedError("TODO human: <pin/urządzenie z rejestru>")`. Nie zmieniaj istniejącej treści `real.py` bez zadania, które tego wprost wymaga
- Fałszywa implementacja **nie jest atrapą zwracającą zera**. Symuluje świat na tyle, żeby logika mogła się pomylić: czujnik przed ścianą zwraca dystans, ruch zmienia pozę, kolizja ustawia flagę
- Interfejsy nazywaj od funkcji, nie od modelu układu: `Wheels`, `Sonar`, `Battery`, nie `PCA9685Driver`, `HCSR04`

## Rejestr sprzętu

- Fakty o sprzęcie (piny, adresy, zasięgi, napięcia, wymiary, prędkości) siedzą w **jednym pliku danych** (np. `hardware/components.yaml`) poza `spec/`, bo zmienia się, gdy ktoś coś przelutuje
- Kod czyta rejestr przez jeden loader (np. `hardware.py`) z walidacją modelu (pydantic): brak wymaganego pola to błąd przy imporcie, nie `None` w runtime
- **Żadnych liczb sprzętowych w kodzie ani w symulatorze.** Prędkość, zasięg sonaru, obrys, kąt wiązki pochodzą z rejestru. Jeśli w kodzie pojawia się `400` albo `0x40`, to błąd
- Wartość oznaczona jako niezweryfikowana (np. `verify: true`) jest prawdziwa dla kodu, ale nie buduj na niej niczego, czego nie da się zmienić jedną edycją rejestru
- Komponent oznaczony jako nieużywany (`used: false`) nie dostaje kodu. Sprzęt w szufladzie nie jest wymaganiem
- Sekcja pułapek w rejestrze to dane wejściowe do `## Znane problemy i rozwiązania` w PROJECT_CONTEXT.md: przepisz ją tam przy pierwszym zadaniu
- Przy każdym zadaniu z tym tagiem: `Read` rejestru sprzętu przed implementacją. Rejestru nie ma w SPEC_INDEX, bo nie jest częścią zamrożonego specu

## Symulator jest sędzią, nie dodatkiem

- **Jeden model świata** (np. `sim/world.py`). Testy, CLI, scenariusze i podgląd w przeglądarce używają tej samej klasy. Podgląd niczego nie liczy sam i niczego nie edytuje
- Świat 2D wystarcza: poza `(x, y, theta)`, prostokątny obrys z rejestru, ściany jako odcinki, czujnik odległości jako promień z pozycji czujnika, kolizja zatrzymuje i ustawia flagę
- **Sztuczny zegar.** `World.step(dt)` przesuwa czas; w symulacji nigdy `time.sleep()` ani `time.time()`. Pętla główna dostaje zegar przez parametr, żeby ten sam kod działał na sztucznym i prawdziwym
- Szum czujników jako parametr świata (domyślnie 0, żeby testy były deterministyczne). Opóźnienie decyzji (np. czas odpowiedzi modelu językowego) też jest parametrem świata, nie pomijaj go
- Każdy scenariusz (`sim/scenarios/*.yaml`) to osobny test pytest przez parametrize po plikach. Runner zwraca kod wyjścia 0 tylko, gdy wszystkie przeszły, i wypisuje, które oczekiwanie padło i z jaką wartością
- Zadanie zmieniające zachowanie robota bez scenariusza jest **niezweryfikowane**, choćby testy jednostkowe były zielone
- Zielony symulator znaczy „logika jest spójna z opisem sprzętu", nie „działa". Odwrotnie podłączony silnik, brownout zasilania, sprzężenie mikrofonu z głośnikiem, dryf enkodera symulator nie wykryje. To trafia do dokumentacji dla ludzi i do scenariuszy ręcznych przy wydaniu (sekcja 15.7), nie do testów

## Bezpieczeństwo ma dolne limity w kodzie

- Progi bezpieczeństwa (minimalny dystans stopu, maksymalny czas ruchu, próg napięcia) mają **minimum zapisane w kodzie** (np. `limits.py`), którego żadna reguła, konfiguracja ani model językowy nie obejdzie. Reguła z wartością poniżej limitu nie przechodzi walidacji przy starcie, z czytelnym komunikatem
- To są jedyne liczby o zachowaniu, które żyją w kodzie. Wszystko inne to dane
- **Każdy ruch ma czas.** Komenda jazdy bez limitu czasu nie istnieje w interfejsie; po upływie czasu koła stają bez udziału logiki wyższej warstwy
- **Watchdog.** Utrata pętli sterowania, utrata połączenia z warstwą decyzyjną, wyjątek w cyklu = stop kół. Sterownik silników trzyma ostatnie PWM po śmierci procesu, więc stop musi być jawny i musi być pierwszą rzeczą w `finally`
- Dystans stopu liczy się z fizyki, nie z intuicji: `prędkość_max × (1/częstotliwość_czujnika + opóźnienie_decyzji) + droga_hamowania`. Przy 40 cm/s i 10 Hz to 4 cm na próbkę; próg 10 cm jest graniczny, 20 cm sensowny. Zapisz wyliczenie w komentarzu przy limicie
- Decyzje z modelu językowego przechodzą przez te same limity i tę samą walidację, co reguły. **Model proponuje, kod filtruje**
- Miękkie limity sprzętowe z rejestru (np. `pwm_max_percent`) egzekwuje `hal/`, nie logika wyżej. Zmiana limitu wymaga zgody człowieka, nie zadania

## Zachowania to dane, nie kod

- Reakcje robota (warunek na czujniku → akcje) są plikami danych (np. `behaviors/*.yaml`), które kod wykonuje w każdym cyklu. Kod dostarcza słownik warunków i akcji, człowiek układa z niego zdania
- Warunek to proste wyrażenie `pole operator wartość`, łączone spójnikami. **Nie używaj `eval`** ani `exec`. Napisz mały parser z jawną listą pól i operatorów; nieznane słowo to błąd walidacji z podpowiedzią najbliższego znanego
- Konflikt akcji rozstrzyga jawny priorytet. Reguły bezpieczeństwa mają najwyższy i są w osobnym pliku
- Każda odpalona reguła loguje: nazwę, wartości warunku, wykonane akcje. To dane dla panelu „dlaczego" i dla scenariuszy (`oczekiwanie: zadziałała_reguła`)
- Gdy zadanie wymaga zmiany kodu zamiast danych, brakuje słowa w słowniku. Dodaj słowo, opisz je w dokumentacji słownika dla ludzi, i dopiero wtedy napisz regułę. Nigdy nie wpisuj zachowania na sztywno w kodzie „bo tak szybciej"
- Słownik dla ludzi (np. `docs/RULES.md`) aktualizuj w tym samym zadaniu, w którym dodajesz słowo. Bez tego słowo nie istnieje dla autora reguł, który nie czyta kodu
- Kanoniczne nazwy słów po angielsku w silniku; aliasy w języku użytkownika w osobnym pliku mapowań, jeśli projekt tego wymaga. Parser normalizuje przed walidacją

## Proces

- **Agent nigdy nie wgrywa kodu na robota.** Skrypt deployu uruchamia człowiek; żaden test, `Makefile`, CI ani skrypt pomocniczy go nie woła. Test: `grep` po `.github/` i `Makefile` nie znajduje nazwy skryptu deployu
- Skrypt deployu ma `--dry-run` i pyta o potwierdzenie przed restartem usługi na robocie
- Nowa zależność tylko z zadania, które ją wymienia. Potrzeba czegoś więcej to `RALPH BLOCKED`, nie `pip install`
- Kod działa na macOS, Windows i Linux, bo robota programują ludzie z laptopów. Skrypty pomocnicze w Pythonie, nie w bashu. CI na trzech systemach
- Identyfikatory w kodzie po angielsku. Język dokumentacji dla ludzi, komunikatów walidacji i komentarzy określa projekt
- Sekrety (klucze API, hasła do robota) w `.env`, wzór w `.env.example`. Adres robota też jest konfiguracją, nie literałem

## Platforma (Raspberry Pi i podobne)

- Osobne zasilanie komputera i silników. Wspólne resetuje komputer przy rozruchu silników. Zapisz to w rejestrze i w instrukcji montażu
- Kolejność pierwszego uruchomienia idzie od najmniej ryzykownego: nakładki bez zasilania silników (`i2cdetect` pokazuje wszystkie adresy) → zasilanie bez silników → jedna para kół → reszta. Kierunek każdego koła sprawdzany osobno, bo silniki bywają zamontowane obrotem o 180°
- Napięcia logiki: czujnik na 5 V bez dzielnika uszkodzi GPIO 3,3 V. Wariant układu (np. HC-SR04 kontra HC-SR04P) to pole w rejestrze, nie wiedza w głowie
- Urządzenia audio przez nazwy ALSA z konfiguracji, nie numery kart (zmieniają się po restarcie). Nie wymuszaj częstotliwości próbkowania, której urządzenie nie obsługuje; wykryj natywną i resampluj
- Biblioteki sprzętowe z pakietów systemowych: venv z `--system-site-packages`, a `real.py` importuje je leniwie, żeby `fake` działał na laptopie bez nich
- Robot jako usługa `systemd --user` z restartem; logi przez `journalctl`. Instrukcja przygotowania systemu dla człowieka w `deploy/`

## Czego unikać

- Importów sprzętowych poza `hal/`
- Liczb sprzętowych w kodzie i symulatorze
- `time.sleep()` w logice sterowania i w symulacji
- `eval` w parserze reguł
- Zachowań zaszytych w kodzie zamiast w danych
- Komendy ruchu bez limitu czasu
- Mocków sprzętu w testach zamiast fałszywego HAL na modelu świata (mock nie pozwoli logice się pomylić)
- Wywoływania deployu z czegokolwiek poza ręką człowieka
- Kodu pod sprzęt oznaczony jako nieużywany
- Zmiany `real.py` i limitów dolnych bez wprost nazwanego zadania
