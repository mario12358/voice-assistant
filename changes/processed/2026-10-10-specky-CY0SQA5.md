---
data: 2026-10-10
rodzaj: rozwój
priorytet: normalny
dotyczy: ogólne (nowe wymaganie Specky, sekcja „Nagrywanie i schowek”)
specky_req: 01M4K06AGAV8G79RJMXCY0SQA5
specky_hash: d3eecdb
---

# VA-HIST-1 — historia wypowiedzi (trwała, z podmenu „Historia”)

Nowe wymaganie zaakceptowane przez właściciela 2026-10-10 (wsad 01M4JZS73RFCBAXE91TSTJCG2V),
powód z kolejki: `to_build` (status `open`).

**Treść (v1 @d3eecdb):** Aplikacja przechowuje historię wypowiedzi: każda niepusta transkrypcja
trafia na listę ostatnich wpisów dostępną z menu ikony w pasku menu, skąd użytkownik może ponownie
skopiować wcześniejszy tekst do schowka. Historia jest zapisywana na dysku w katalogu danych
aplikacji i wraca po ponownym uruchomieniu.

**Kryteria:**
1. (crit: 01M4K06AH032985T6JXJX7EE3W) Po każdej niepustej transkrypcji w historii pojawia się nowy wpis z pełnym tekstem i czasem; pusta transkrypcja (sama cisza) nie tworzy wpisu
2. (crit: 01M4K06AH0CZS3Z7N0FXXSKE1T) Menu ikony zawiera podmenu „Historia” z ostatnimi wpisami od najnowszego: każda pozycja pokazuje godzinę i początek tekstu; lista ma górny limit (30 wpisów), najstarsze wypadają
3. (crit: 01M4K06AH09HZHH764X48P4GM4) Kliknięcie wpisu w podmenu „Historia” kopiuje jego pełny tekst do schowka systemowego (wklejany cmd+v), bez uruchamiania nagrania
4. (crit: 01M4K06AH0FMZJZZX7SHA9KR5A) Historia jest zapisana w ~/Library/Application Support/VoiceAsystent/ w pliku dostępnym tylko dla bieżącego użytkownika i po ponownym uruchomieniu aplikacji podmenu pokazuje te same wpisy
5. (crit: 01M4K06AH0BR7ZQYXPH6B08P26) Pozycja „Wyczyść historię” usuwa wszystkie wpisy z menu i z dysku
6. (crit: 01M4K06AH0B2HW26D29WR4H4F3) Treść wpisów historii nie trafia do logów aplikacji

**Kontekst z rozmowy z właścicielem (2026-10-10):** „schowek na wypowiedzi”, żeby wrócić do
wcześniejszych transkrypcji; właściciel wybrał wariant trwały (zapis na dysku) zamiast samej pamięci.
To jedyny dozwolony zapis treści transkrypcji na dysk — logi dalej bez treści (decyzja 📌
w PROJECT_CONTEXT). Logika historii ma żyć w `va-core` (kontroler), UI tylko pokazuje podmenu
i wysyła polecenia, jak przy mikrofonach.
