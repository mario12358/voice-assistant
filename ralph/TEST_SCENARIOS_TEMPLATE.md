# vX.Y.Z — scenariusze testów manualnych

<!--
Szablon pliku tworzonego przy KAŻDYM tagu wersji (sekcja 15.7 instrukcji Ralpha).
Nazwa pliku = nazwa taga: docs/test-scenarios/vX.Y.Z.md
Zbiera WYŁĄCZNIE to, czego testy automatyczne sprawdzić nie mogły.
Szablon jest stackowo neutralny — komendy i nazwy pakietów bierz z docs/RUNBOOK.md
oraz z pola `Testy` w ralph/config.md.
Usuń ten komentarz i wszystkie nawiasy <...> przy wypełnianiu.
-->

- **Tag:** `vX.Y.Z`
- **Data:** <RRRR-MM-DD>
- **Poprzednia wersja:** `<vX.Y-1.0>`
- **Zakres:** <fazy / obszary, po ludzku — nie numery zadań>
- **Migracje / kroki wdrożeniowe:** <co trzeba uruchomić przy wdrożeniu + jednozdaniowy skutek, albo „brak”>
- **Testy automatyczne:** <wynik suite'u z pola `Testy` w ralph/config.md — liczby per pakiet>, zielone. Poniżej wyłącznie to, czego automat sprawdzić nie mógł.

**Oznaczanie wyniku:** `[x]` = sprawdzone i działa · `[-]` = sprawdzone i NIE działa · `[ ]` = niesprawdzone.
**Jeden scenariusz = jeden checkbox** (linia `Wynik N.M` na końcu scenariusza). Przy `[-]` dopisz pod
scenariuszem jedno zdanie: co zobaczyłeś zamiast oczekiwanego.

---

## 0. Przygotowanie

- [ ] <komenda migracji ze stacku projektu — patrz docs/RUNBOOK.md> kończy się stanem `<oczekiwany>`
- [ ] Kopia bazy zrobiona (jeśli wydanie zmienia dane, nie tylko schemat)
- [ ] Konto o wymaganej roli: <rola>
- [ ] Dane, bez których scenariusze nic nie pokażą: <np. projekt sprzed zmiany, rozmowa > N wiadomości>
- [ ] Środowisko i jego różnice wobec produkcji: <np. inny dostawca modelu>

---

## 1. <Obszar — nazwa, którą rozpozna user, nie nazwa modułu>

### 1.1 <Co sprawdzamy — zdaniem>

**Dlaczego:** <jedno zdanie: co było zepsute albo co jest nowe>

**Kroki:**
1. <klikalny krok>
2. <klikalny krok>

**Oczekiwane:** <konkret, po czym poznasz sukces>
**Sygnał porażki:** <co konkretnie znaczy, że zmiana nie zadziałała>
**Wzorzec:** <ścieżka do artifacts/flows/krok-N-<slug>.<ext>, gdy jest nagranie tej ścieżki — inaczej usuń tę linię>
**Wynik pomiaru:** ____ <tylko gdy kryterium akceptacji mówi o liczbie, której nie zmierzyły testy — inaczej usuń tę linię>

- [ ] **Wynik 1.1** — zgodne z „Oczekiwane”

---

## <N>. Bariery — te scenariusze mają się NIE udać

<!-- Sekcja obowiązkowa, gdy wydanie dokłada ograniczenie, walidację albo uprawnienie. -->

### <N>.1 <Próba, która ma zostać odrzucona>

**Kroki:** <żądanie / akcja, najlepiej z pominięciem interfejsu>

**Oczekiwane:** <kod błędu, odmowa, brak zapisu w bazie>
**Sygnał porażki:** <cokolwiek, co przechodzi>

- [ ] **Wynik <N>.1** — próba odrzucona zgodnie z „Oczekiwane”

---

## <N+1>. Regresja obszarów dotkniętych pośrednio

<!--
Ścieżki, które nie były celem zmiany, ale przechodzą przez zmieniony kod.
Gdy projekt ma spec/APP_FLOW.md — wskaż Kroki po nazwie zamiast opisywać ścieżkę od nowa.
-->

- [ ] <Krok N — Tytuł (spec/APP_FLOW.md)>
- [ ] <ścieżka bez odpowiednika w APP_FLOW — jednym zdaniem>

---

## Wynik

- **Testował:** _______
- **Data:** _______
- **Środowisko:** dev / produkcja
- **Scenariusze nieudane (`[-]`):** _______
- **Decyzja:** wersja gotowa do wdrożenia / wymaga poprawek
