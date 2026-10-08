# Mapa powierzchni ataku

<!-- Deklaracja intencji: kto ma dostęp do czego. Kontrola `intencja` porównuje z nią kod przy     -->
<!-- każdym commicie, kontrola `red-team` — testy na koniec fazy. Plik pisze Ralph (nowa trasa =    -->
<!-- nowy wiersz w tym samym commicie), człowiek widzi zmiany w propozycji wydania i w diffie PR.  -->
<!-- Nie idzie do prompta startowego.                                                               -->

- **Sesja**: ?
  <!-- ciasteczko | nagłówek — jak przeglądarka przenosi tożsamość. ciasteczko (albo ?) = każda     -->
  <!-- trasa zapisu potrzebuje przypadku red teamu `csrf`; nagłówek (Bearer) = nie potrzebuje.      -->

## Role

<!-- Rola = kto (ze spec). Strażnik w kodzie = tekst, który w oknie definicji trasy (dekoratory i   -->
<!-- sygnatura, argumenty przed handlerem, @UseGuards, middleware z prefiksem) oznacza tę rolę.     -->
<!-- Kilka strażników po przecinku = którykolwiek wystarcza. `(globalny)` = strażnik stoi poza      -->
<!-- trasą (middleware całej aplikacji, proxy forward_auth) — kod trasy nie jest wtedy sprawdzany,  -->
<!-- zostaje red team. `anonim` jest wbudowana: trasa publiczna.                                     -->

| Rola | Kto | Strażnik w kodzie |
|---|---|---|
| anonim | bez sesji | — |

## Granice własności

<!-- Czego nie sprawdzi żadna reguła: czy lekarz A widzi pacjenta lekarza B, czy członek projektu  -->
<!-- X zmienia projekt Y. Każda trasa z kolumną Własność ≠ — dostaje w red teamie przypadek         -->
<!-- `obcy-wlasciciel`. Wiersz = zasób, kto może, gdzie jest granica i co zwraca naruszenie.         -->

| Zasób | Kto może | Granica |
|---|---|---|

## Trasy

<!-- Role: rola albo `a + b` (wymagane obie); alternatywy po przecinku. `?` = nieustalone           -->
<!-- (ostrzega, dopóki ktoś nie zdecyduje: publiczna czy brakuje strażnika). Własność: zasób z       -->
<!-- tabeli wyżej, `—` = brak, `?` = nieustalone (red team traktuje jak zasób).                      -->

| Trasa | Role | Własność | Plik |
|---|---|---|---|

## Wyjścia

<!-- host = domena, do której kod wysyła żądania; env = zmienna środowiskowa z sekretem;             -->
<!-- baza = fragment polecenia nadającego uprawnienia (GRANT, SECURITY DEFINER, BYPASSRLS…).         -->
<!-- Nowa wartość w kodzie bez wiersza tutaj = ostrzeżenie.                                          -->

| Rodzaj | Wartość | Po co |
|---|---|---|
