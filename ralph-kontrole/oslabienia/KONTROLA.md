---
rodzaj: regula
uruchom: python3 run.py
kiedy: commit, faza, wydanie, ci
zakres: zmienione
przy_bledzie: blokuje
---

# Kontrola: osłabienia

**Osłabienie** to miejsce, w którym zabezpieczenie zostało świadomie wyłączone albo
obejście dodane „na chwilę": kod TOTP pokazywany pod formularzem logowania, wyłączona
weryfikacja TLS, tryb debug, obejście logowania do testów ręcznych, mutant z testu
mutacyjnego. Bywa potrzebne — ale nie może dotrzeć do wydania niezauważone.

## Znacznik

Każde osłabienie dostaje w kodzie, w tej samej albo poprzedniej linii, komentarz:

```
ralph: osłabienie <id> — <powód i warunek usunięcia>
```

np. `# ralph: osłabienie totp-podglad — kod TOTP pod formularzem, właściciel stracił
authenticator; usunąć przed wdrożeniem`. Znacznik jest rejestrem: lista otwartych osłabień
to `grep -rn "ralph: osłabienie"`.

## Co blokuje, a co ostrzega

| Punkt | Blokuje | Ostrzega |
|---|---|---|
| commit | znacznik bez id albo bez powodu; `MUTANT` w kodzie | wzorzec osłabienia **bez** znacznika (TLS off, debug, `csrf_exempt`, `skip_auth`…) |
| faza | `MUTANT` | każde otwarte osłabienie (przypomnienie) + wzorce bez znacznika |
| wydanie | **każde otwarte osłabienie**, `MUTANT` | wzorce bez znacznika |

Wzorce w plikach testów (`tests/`, `*_test.*`, `*.spec.*`, `test_*`) są pomijane — testy
legalnie robią to wszystko. `MUTANT` pomijany nie jest: mutant zostawiony w teście też
oznacza, że ktoś przerwał test mutacyjny w połowie.

Osłabienie usuwa się razem ze znacznikiem. Wydanie z osłabieniem, które ma zostać (np.
środowisko demo), wymaga wyjątku po odcisku w `ralph/KONTROLE_WYJATKI.md` — wpisuje człowiek.
