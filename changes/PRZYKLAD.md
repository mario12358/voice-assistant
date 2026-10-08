---
data: YYYY-MM-DD
priorytet: normalny
rodzaj: rozwój
dotyczy: spec/<plik>.md (sekcja "<nazwa sekcji>") | spec/<plik>.md:<linia-od>-<linia-do> | ogólne
---

<!--
Plik zmiany — wprowadza modyfikację specyfikacji po fazie doprecyzowania.

Jak używać:
1. Skopiuj ten plik z dowolną nazwą do katalogu changes/ w projekcie
   (np. changes/2026-04-23-walidacja-email.md). Nazwa dowolna.
2. Wypełnij frontmatter (opcjonalny, ale zalecany):
   - data: data zgłoszenia zmiany
   - priorytet: krytyczny | normalny (krytyczne aplikowane jako pierwsze)
   - rodzaj: rozwój (nowy zakres) | korekta (działa jak ustalono, ma działać inaczej)
     | błąd (coś zbudowanego nie działa). Brak pola → Ralph oceni sam po treści.
   - dotyczy: wskaźnik na sekcję spec/ której zmiana dotyczy
     (Ralph wczyta tę sekcję żeby porównać "było vs jest")
3. Opisz w treści CO się zmienia i DLACZEGO — bez tego Ralph zablokuje aplikację.
4. Zapisz plik. Ralph przetworzy go przy następnym uruchomieniu:
   - Zmodyfikuje ralph/PLAN.md (zadania pending lub utworzy fazę rework "N.Z1" dla ✅)
   - Doda wpis do ralph/REPORT.md → ## Historia zmian
   - Przeniesie ten plik do changes/processed/ (audyt w gicie)

Co Ralph robi z różnymi typami zmian:
- Pending zadanie → modyfikuje opis w miejscu + adnotacja (zmiana: <nazwa-pliku>)
- Nowa funkcjonalność → dopisuje nowe zadania do odpowiedniej fazy
- Zmiana dotyka ✅ ukończonego zadania → tworzy nową fazę rework (np. Faza 6.Z1)
  z zadaniami `(wymaga: <oryginalny-numer>)`. Checkbox ✅ NIE jest odznaczany.

Usuń tę sekcję komentarza po wypełnieniu.
-->

# Co się zmienia

[Opisz konkretnie jaką funkcjonalność/zachowanie/wymaganie zmieniamy.
Bądź jednoznaczny — jeśli treść jest niejasna, Ralph zwróci RALPH BLOCKED zamiast zgadywać.]

# Dlaczego

[Powód zmiany: zgłoszenie od klienta, decyzja produktowa, wymóg compliance, wykryty bug
w specyfikacji, zmiana priorytetów, itp. To pomoże przyszłemu czytelnikowi historii zmian
zrozumieć kontekst.]

# Wpływ (opcjonalnie)

[Jeśli wiesz które zadania w ralph/PLAN.md są dotknięte — wymień je tutaj.
Ralph i tak przeanalizuje plan, ale Twoja podpowiedź przyspiesza i zmniejsza ryzyko pomyłki.

Przykład:
- Faza 3 zadanie 3.2 (LoginForm) — wymaga rework, dodać sprawdzenie MX
- Faza 5 zadanie 5.1 (RegisterForm) — pending, zaktualizować opis
- Nowe zadanie: walidator MX po stronie backendu, do Fazy 2]
