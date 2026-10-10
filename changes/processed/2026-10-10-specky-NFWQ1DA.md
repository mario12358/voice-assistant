---
data: 2026-10-10
rodzaj: rozwój
priorytet: normalny
dotyczy: ogólne (nowe wymaganie Specky, sekcja „Rozpoznawanie mowy”; kod: tray_menu.rs, download.rs, startup.rs, app.rs, crates/model)
specky_req: 01M4K6M2ZC8PQ1AE9FQNFWQ1DA
specky_hash: b18086a
---

# VA-MODEL-2 — podmenu „Model”: informacja o modelu, Pokaż w Finderze, Usuń, Pobierz ponownie

Nowe wymaganie zaakceptowane przez właściciela 2026-10-10 (wsad 01M4K6JNNT5ZPGGVCXY8D7AMAP),
powód z kolejki: `to_build` (status `open`).

**Treść (v1 @b18086a):** Menu ikony ma podmenu „Model” pokazujące, który model rozpoznawania
mowy jest używany (nazwa, rozmiar pliku, stan: gotowy / pobieranie / brak), z poleceniami „Pokaż
w Finderze”, „Usuń model” (po potwierdzeniu; zwalnia miejsce na dysku, nagrywanie staje się
niedostępne do ponownego pobrania) i „Pobierz ponownie”.

**Kryteria:**
1. (crit: 01M4K6M2ZYSP1Y9M0VD39GV7J8) Podmenu „Model” pokazuje nieklikalną pozycję z nazwą modelu (large-v3-turbo), rozmiarem pliku w GB i stanem: „gotowy”, „pobieranie N%” albo „brak”; pozycja aktualizuje się w trakcie pobierania
2. (crit: 01M4K6M2ZY9XTRRVSGXYM05D1P) „Pokaż w Finderze” otwiera katalog z plikiem modelu w Finderze
3. (crit: 01M4K6M2ZYTC7D38NMHZJXF1FV) „Usuń model” pyta o potwierdzenie, po nim kasuje plik modelu i ewentualny plik częściowy; menu pokazuje „brak”, start nagrywania daje komunikat o braku modelu, a „Pobierz ponownie” uruchamia pobieranie z postępem jak przy pierwszym uruchomieniu
4. (crit: 01M4K6M2ZYHSYT6K38YWRG54RT) Po pobraniu po „Usuń model” nagrywanie działa bez ponownego uruchamiania aplikacji
5. (crit: 01M4K6M2ZYCYED1VTTXDC6R542) „Usuń model” jest nieaktywne, gdy model jest w trakcie pobierania albo transkrypcja trwa

**Kontekst z rozmowy (2026-10-10):** właściciel chciał widzieć wybrany model i móc go usunąć /
pobrać ponownie; wybrał wariant informacyjny (bez przełączania modeli — VA-STT-1 bez zmian).
Dziś model jest w stałym katalogu, menu pokazuje tylko status pobierania i „Ponów pobieranie”
po błędzie; usunięcie modelu = ręczne `rm`. Potwierdzenie usunięcia robimy w samym menu
(pozycja zmienia się w „Potwierdź usunięcie” + „Anuluj”), bez natywnych okien dialogowych
(brak dodatkowych zależności i `unsafe`). Po usunięciu kontroler jest zamykany (model
zwolniony), a „Pobierz ponownie” składa go od nowa jak przy pierwszym uruchomieniu (5.6).
