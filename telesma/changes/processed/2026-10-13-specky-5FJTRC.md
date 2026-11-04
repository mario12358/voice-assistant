---
data: 2026-10-13
rodzaj: korekta
dotyczy: zadanie 5.1 (apps/voice-asystent/src/indicator.rs), VA-UI-1
specky_req: 01M4EKHD1BQYRNWBAVRA5FJTRC
specky_hash: 571b8a6
---
# VA-UI-1 v2 — żółte kółko w trakcie transkrypcji

Powód z kolejki: `to_fix` po zmianie treści wymagania po `code_ready` (`needs_reimplementation=false`).

Było (v1 @df8032b): szare kółko, gdy aplikacja czeka, czerwone podczas nagrywania; po zatrzymaniu
nagrywania ikona od razu wraca do szarego (także w trakcie transkrypcji).

Jest (v2 @571b8a6): szare — czeka (można wklejać ostatnią transkrypcję), czerwone — nagrywanie,
**żółte — trwa transkrypcja**.

Kryteria v2:
1. (01M4EKHD1PEEATZBYQYNAE6NGM) Po uruchomieniu aplikacji w pasku menu widoczne jest szare kółko — bez zmian.
2. (01M4EKHD1PF30TMM4X3RP8Q5MY) Po rozpoczęciu nagrywania (skrót lub kliknięcie) ikona zmienia się na czerwone kółko — bez zmian.
3. (01M4EKHD1PJ32E01218FSE9G8F, zmienione) Po zatrzymaniu nagrywania (skrót, kliknięcie albo limit długości) ikona zmienia się na żółte kółko i pozostaje żółta przez cały czas transkrypcji.
4. (01M4TFDKBW8V3E5M743BZ6JXN2, nowe) Po zakończeniu transkrypcji ikona wraca do szarego kółka — także gdy nagranie było samą ciszą albo transkrypcja zakończyła się błędem; szare po żółtym oznacza, że tekst jest już w schowku.
5. (01M4TFDKGRFC1CYWGCXA5E8HQR, nowe) Podczas transkrypcji (żółte kółko) kliknięcie ikony i ctrl+cmd+r nie rozpoczynają nowego nagrania.

Stan kodu: `indicator_for(State::Transcribing)` daje dziś `Dot::Gray` (test
`after_recording_the_dot_is_gray_again` sprawdza to wprost) — trzeba dodać `Dot::Yellow`.
Kryterium 5 kod już spełnia (`click.rs`: Transcribing → brak polecenia; automat ignoruje Start
w Transcribing), brakuje testów ze znacznikami. Kryterium 4: kolejność — ikona szara dopiero po
zapisie do schowka (zdarzenie stanu Idle po `Delivered`), także po pustej transkrypcji i błędzie.
