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
