---
rodzaj: regula
uruchom: python3 run.py
sprawdz: python3 run.py --sprawdz
kiedy: faza, wydanie, ci
zakres: zmienione
przy_bledzie: blokuje
---

# Kontrola: red-team

W projekcie medycznym realną dziurę w autoryzacji („lekarz A1 przyjmował pacjenta z kolejki A2")
znalazł red team z mutacją — nie testy implementacji, które były zielone. Ralph wymyślił tam
red team per rola na koniec każdej fazy sam; ta kontrola robi z tego regułę dla każdego
projektu z mapą powierzchni ataku (`ralph/BEZPIECZENSTWO.md`, kontrola `intencja`).

**Co sprawdza.** Trasy z mapy o roli innej niż `anonim`, których definicja albo ciało zmieniły
się od poprzedniego tagu fazy (przy `wydanie` — od poprzedniej wersji, w CI — od `RALPH_OD`).
Każda musi mieć w którymś pliku testów znacznik:

```
# ralph: red-team PATCH /projects/{project_id} — bez-sesji, obca-rola, obcy-wlasciciel, csrf
```

Parametr ścieżki w dowolnej formie (`{id}`, `:id`, `<id>`), ale jako parametr — `/projects/5` nie
pasuje do `/projects/{id}`; prefiks montowania (`/api`) może być w znaczniku albo nie. Kilka znaczników dla jednej trasy
sumuje przypadki. Wymagane przypadki wynikają z mapy:

| Przypadek | Kiedy wymagany | Co test robi |
|---|---|---|
| `bez-sesji` | zawsze | żądanie bez uwierzytelnienia → 401/403/przekierowanie, stan bez zmian |
| `obca-rola` | mapa ma co najmniej dwie role poza `anonim`, a trasa nie dopuszcza wszystkich | konto roli spoza listy trasy → 403 |
| `obcy-wlasciciel` | kolumna Własność ≠ `—` (także `?`) | konto z właściwą rolą, ale cudzy zasób (inna organizacja, inny pacjent) → 403/404 |
| `csrf` | metoda zapisu i `Sesja` ≠ `nagłówek` | zapis bez nagłówka Origin i z obcym → 403, stan bez zmian |

Aliasy w znaczniku: `inna-organizacja`, `obca-organizacja` = `obcy-wlasciciel`.

Test iterujący po liście tras modułu („CSRF na każdej trasie zapisu rejestracji") oznacza się
jednym znacznikiem z wieloznacznikiem: `ralph: red-team * /registrar/* — csrf` (metoda `*` albo
`ALL`, ścieżka zakończona `/*` = każda trasa pod prefiksem). Wieloznacznik deklaruje więcej niż
pojedyncza linia — tym bardziej test musi faktycznie przejść po wszystkich trasach z listy.

| Reguła | Waga | Co |
|---|---|---|
| `red-team-brak` | **blokuje** | chroniona trasa zmieniona w fazie bez któregoś z wymaganych przypadków. Opis to gotowa treść zadania: trasa, brakujące przypadki, linia znacznika do wpisania |
| `red-team-nieustalone` | ostrzega | trasy zmienione w fazie z rolą `?` w mapie — nieobjęte, dopóki rola nie zostanie ustalona |

**Pierwsza faza po założeniu mapy tylko ostrzega** (mapy nie ma w punkcie odniesienia): role ze
szkieletu nikt jeszcze nie przejrzał, a blokada tagu przy pierwszym starcie zatrzymałaby pętlę
na długu sprzed kontroli. Od następnej fazy blokuje.

**Czego nie sprawdza.** Że test faktycznie atakuje i że padłby, gdyby strażnika usunąć — znacznik
to deklaracja. Sprawdza to test mutacyjny z `ralph-guidelines/general.md` (usuń strażnika, test
musi zrobić się czerwony) i audytor fazy (etap E4). Ta kontrola pilnuje, że test w ogóle powstał
dla każdej zmienionej trasy i każdego przypadku, który wynika z mapy.

**Pomijane:** trasy publiczne, trasy bez wiersza w mapie (zgłasza je `intencja`), pliki testów
jako źródło tras.
