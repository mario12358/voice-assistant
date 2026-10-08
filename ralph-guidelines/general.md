---
tag: general
keywords: clean code, naming, comments, testing, git, refactoring, errors, shell, komendy
---

# Wytyczne ogólne (zawsze ładowane)

## Clean code

- Nazwy opisowe, intencjonalne — unikaj skrótów (`userById`, nie `ubi`)
- Funkcje robią jedną rzecz, krótkie (≤ 20 linii)
- Zagnieżdżanie > 3 poziomów — ekstraktuj do funkcji
- Preferuj early return zamiast else
- W łańcuchach i nazwach testów nie używaj cudzysłowu typograficznego `„…"` — zamykający bywa zapisywany jako ASCII `"` i urywa string (błąd składni, którego kompilator typów nie pokaże, a komunikat transformacji wskazuje gdzie indziej). Cytat w kodzie: apostrofy albo `«…»`

## Komentarze

- Domyślnie nie pisz komentarzy — kod powinien być samodokumentujący
- Komentuj TYLKO "dlaczego" gdy nieoczywiste, nigdy "co robi kod"
- Usuwaj martwy kod zamiast komentować (git pamięta)

## Testy

- Test każdej nowej funkcjonalności: happy path + minimum jeden edge case
- Test nowego serwisu wołanego wprost nie dowodzi, że cokolwiek go woła — do każdego nowego mechanizmu jeden test przez miejsce wpięcia (endpoint, handler, ekran, worker)
- Dane testowe w kształcie produkcyjnym, nie najprostszym: jeśli encja w praktyce występuje w grupach/partiach/wielu wersjach, test na pojedynczej sztuce omija całą gałąź
- Nazwy testów opisują zachowanie, nie implementację
- Izoluj zależności zewnętrzne (baza, API, czas) — mock lub fixture
- Failujący test naprawiaj zmianą kodu, nie testu — osłabienie asercji/skip tylko gdy test jest błędny względem wymagań (z uzasadnieniem i wpisem do logu problemów)
- Wynik "0 testów wykonanych" to błąd konfiguracji, nie sukces — nawet przy exit code 0
- Test REGUŁY (walidacja, uprawnienie, limit, próg, bariera) sprawdź mutacją, zanim uznasz go za gotowy: wyłącz regułę w kodzie produkcyjnym i uruchom test — ma paść. Zielony pod mutacją = test niczego nie pilnuje (najczęściej: dane testowe nie wchodzą w gałąź, asercja liczy elementy zamiast wskazać konkretny, scenariusz nie odróżnia dwóch reguł)
- Procedura mutacji, bez wyjątków: (1) mutuj JEDNĄ linię, oznacz ją komentarzem `MUTANT` w osobnej linii NAD zmianą, nie wewnątrz badanego wyrażenia; (2) cofaj odwrotną edycją, NIGDY `git checkout`/`git restore` — na pliku z niezacommitowaną pracą to kasowanie, nie cofanie; (3) test uruchamiany pod mutacją nie może pisać do plików repozytorium — wyjście kieruj do katalogu tymczasowego; (4) po cofnięciu uruchom test jeszcze raz — zielony jest dowodem, że mutant zniknął

## Błędy

- Nie tłum wyjątków cicho (żadnych pustych catch)
- Waliduj na granicach (wejście usera, zewnętrzne API) — nie wewnątrz zaufanego kodu
- Nie dodawaj try/catch "na wszelki wypadek"

## Git

- Konkretne `git add <plik>`, nigdy `git add -A` / `git add .`
- Jeden commit = jedna logiczna zmiana
- Wiadomość w trybie rozkazującym ("dodaj X", nie "dodałem X")
- Nie commituj sekretów (.env, klucze API, credentials)

## Komendy powłoki

Claude Code analizuje każdą komendę, żeby dopasować ją do reguł uprawnień. Konstrukcji,
których nie da się jednoznacznie przeanalizować, nie obejmie żadna reguła `allow` — one
zawsze pytają użytkownika, choćby narzędzie było dozwolone. Siedem wzorców psuje to
najczęściej; każdy ma równoważny zapis, który przechodzi bez pytania:

| Zamiast | Napisz | Dlaczego |
|---|---|---|
| `SP=/tmp/x` … `cp $SP/a.bak .` | ścieżkę wprost: `cp /tmp/x/a.bak .` | rozwinięcie zmiennej ukrywa realny cel przed analizą |
| `cd frontend && git add src/a.ts` | `git -C frontend add src/a.ts` | `cd` przed `git` może odpalić hooki z innego katalogu |
| `cd A && … && cd B && …` | osobne wywołania Bash | wielokrotna zmiana katalogu zawsze wymaga zgody |
| `cd ../backend && cp a b` | pełne ścieżki bez `cd` | przy `cd` z zapisem analizator nie wie, do czego odnoszą się ścieżki |
| `for l in pl en de; do … done` | pętla **wewnątrz** skryptu (`python3 - <<PY`) albo osobne wywołania | pętli powłoki analizator nie parsuje; przy okazji gubisz wynik pojedynczego przebiegu |
| `cat >> plik <<'EOF' … EOF` | narzędzie Edit/Write | ta sama zmiana, widoczna jako diff, bez cytowania powłoki |
| `python3 <<PY … p.write_text(s) … PY` | narzędzie Edit/Write | skryptem rób odczyt i analizę; zapis narzędziem — pewniej i z diffem |

Ogólna zasada: komenda ma być **jednym czytelnym wywołaniem o jawnych argumentach**.
Kod (Python, JS) pisz do pliku narzędziem Edit/Write i uruchamiaj plik, zamiast wklejać
go do powłoki przez heredoc albo `-c`.

## Refaktor

- Trzy podobne linie < przedwczesna abstrakcja
- Nie refaktoruj przy naprawie buga (osobne zadanie)
- YAGNI — nie projektuj pod hipotetyczne przyszłe wymagania
- Nie dodawaj flag kompatybilności / fallbacków dla scenariuszy które się nie zdarzą
