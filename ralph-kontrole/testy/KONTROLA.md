---
rodzaj: regula
uruchom: python3 run.py
kiedy: commit, ci
zakres: zmienione
przy_bledzie: blokuje
---

# Kontrola: testy

Zielony suite, w którym po cichu wyłączono część testów, jest gorszy od czerwonego — na czerwony
ktoś patrzy. Retro pokazało, skąd się to bierze: 53 ze 134 wpisów logu problemów to zmiany
istniejących testów (test przypięty do implementacji, nie do reguły), a benchmark postawił 208
zielonych testów Ralpha obok sędziego z zewnątrz. Ta kontrola łapie osłabienie testów przy
commicie, zanim trafi do historii.

## Co blokuje, a co ostrzega

| Reguła | Waga | Co łapie |
|---|---|---|
| `only` | blokuje | `.only(`, `fit(`, `fdescribe(` w pliku testu JS/TS — wyłącza resztę suite'u |
| `skip-bez-powodu` | blokuje | `@pytest.mark.skip` / `skipif` bez `reason=`, `@unittest.skip()` bez tekstu, `it.skip(` / `test.skip(` / `describe.skip(` / `xit(` / `xdescribe(` bez komentarza `// skip:` / `// powód:` / `// reason:` w tej samej linii lub linii wyżej, Go `t.Skip()` bez argumentu, Rust `#[ignore]` bez `= "…"`, Java/Kotlin `@Disabled` / `@Ignore` bez tekstu |
| `skip-z-powodem` | ostrzega | te same konstrukcje z uzasadnieniem — przypomnienie, że suite jest zielony bez tego testu |
| `test-usuniety` | blokuje | plik testu usunięty w tym commicie, gdy testowany moduł (po nazwie: `test_foo.py` ↔ `foo.py`, `Foo.test.tsx` ↔ `Foo.tsx`, `foo.spec.ts` ↔ `foo.ts`, `foo_test.go` ↔ `foo.go`, `FooTest.java` ↔ `Foo.java`) nadal istnieje w repo; moduł znika razem z testem → porządek, przepuszczone |
| `test-bez-asercji` | ostrzega | funkcja testowa (`def test_…`, `it(`, `test(`, `func Test…`) bez `assert` / `expect(` / `should` / `t.Error` / `t.Fatal` / `require.` / `pytest.raises` / `toThrow`; także asercja pusta: `assert True`, `assert 1`, `expect(true).toBe(true)`, `expect(1).toBe(1)` |
| `kod-bez-testu` | ostrzega | w commicie jest plik kodu w katalogu kodu, a nie ma żadnego pliku testu — **jedno** znalezisko na commit, odcisk z listy plików |
| `test-przypiety` | ostrzega | para kod ↔ test (po nazwie) i **oba** modyfikowane (status M); w teście usunięto albo zmieniono linię z treścią testu — jeśli zmiana testu wynika z implementacji, nie z reguły, test pilnuje implementacji. Nowy kod z nowym testem (A + A) to norma; test, do którego tylko dopisano przypadki, też — zmienione importy, puste linie i komentarze się nie liczą (rozszerzony import dał fałszywe ostrzeżenie na Specky) |
| `mniej-asercji` | ostrzega | w zmodyfikowanym pliku testu liczba `assert` / `expect(` spadła względem wersji z bazy |

**Pliki testów** rozpoznaje ta sama reguła co w `oslabienia` (`tests/`, `__tests__/`, `spec/`,
`e2e/`, `test_*`, `*.test.*`, `*.spec.*`, `*_test.*`). Rust `#[ignore]` jest sprawdzany w każdym
`.rs` — testy Rusta siedzą w `#[cfg(test)]` w pliku modułu.

**Katalogi kodu** (`kod-bez-testu`): `src`, `app`, `lib`, `backend`, `frontend`, `server`, `client`,
`api`, `pkg`, `internal`, `cmd`, `packages`, `apps`, `services`, `core`, `domain`, `web`, `mobile`
oraz pliki w korzeniu repo (płaski projekt). Lista do nadpisania w configu: `- **Katalogi kodu**:
src, app` w podsekcji `### testy`. Pomijane jako pliki bez logiki: nazwy od kropki, `*config*`,
`settings.py`, `conftest.py`, `__init__.py`, `setup.py`, `manage.py`, `types.ts` / `constants.py` /
`enums.py`, `*.d.ts`, `index.ts` tylko re-eksportujący, katalogi `migrations/`, `alembic/`,
`scripts/`, `tools/`, `docs/`, `.github/`; Markdown / JSON / CSS / YAML nie są kodem.

## Baza porównania

Przy `commit` bazą jest `HEAD` (status A/M = czy plik jest w HEAD; poprzednia wersja =
`git show HEAD:<plik>`; usunięcia = indeks ∪ drzewo robocze, bo usunięte pliki nie przychodzą
na stdin od runnera). W CI bazą jest merge-base z `RALPH_OD` (gdy ustawione), `origin/main` albo
`origin/master`; bez żadnego z nich reguły zależne od poprzedniej wersji (`test-usuniety`,
`test-przypiety`, `mniej-asercji`) są pomijane — reguły treściowe (`only`, skipy, asercje)
i `kod-bez-testu` działają zawsze.

## Czego nie robi

Nie uruchamia testów i nie czyta wyników — to robi suite. Helper asertujący rozpoznaje tylko po nazwie
(`self.sprawdz_…`, `self.check_…`, `self.verify_…`, `self.blad(…)`) — inny helper da fałszywe
ostrzeżenie; stąd `test-bez-asercji` tylko ostrzega. Nie
rozpoznaje `pytest.skip()` wywołanego w ciele testu (skip warunkowy od środowiska bywa legalny).
Nie ocenia `fit:` jako fokusu (klucz obiektu `{ fit: 'cover' }` w fiksturze dałby fałszywą blokadę).
Parowanie kod ↔ test idzie po nazwie pliku, nie po importach: test `tests/test_api_flow.py`
bez modułu `api_flow.py` nie ma pary i nie wchodzi w `test-przypiety` ani `test-usuniety`.

Fałszywy alarm (np. `.only` w fiksturze, która sama jest tekstem testu) → wyjątek po odcisku
w `ralph/KONTROLE_WYJATKI.md`, wpisuje człowiek. Odcisk `only` / skipów to treść linii bez
białych znaków — przeżywa przesunięcie linii; odcisk `kod-bez-testu` to lista plików, nie miejsce.
