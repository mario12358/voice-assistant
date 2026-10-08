---
rodzaj: komenda
uruchom: python3 run.py
kiedy: faza, wydanie, ci
zakres: całość
przy_bledzie: blokuje
---

# Kontrola: audyt zależności

Znane podatności w zainstalowanych wersjach zależności, według bazy
[OSV](https://osv.dev) (ta sama, z której korzysta `osv-scanner`; zbiera GitHub Advisories,
PyPA, npm i inne). Nie wymaga instalowania żadnego narzędzia.

**Co czyta.** Lockfile'e śledzone w repo — niezależnie od tego, co się zmieniło: podatność
w zależności pojawia się w bazie bez żadnej zmiany w projekcie, więc ta kontrola zawsze
patrzy na całość. Obsługiwane: `package-lock.json`, `npm-shrinkwrap.json`, `pnpm-lock.yaml`,
`yarn.lock`, `deno.lock` (pakiety npm), `poetry.lock`, `uv.lock`, `Pipfile.lock`,
`requirements*.txt` (tylko wersje przypięte `==`).

| Podatność | Waga |
|---|---|
| krytyczna albo wysoka, **jest wersja z poprawką**, zależność produkcyjna | blokuje |
| krytyczna albo wysoka bez poprawki, albo w zależności deweloperskiej | ostrzega |
| średnia, niska, bez oceny | ostrzega |
| baza nieosiągalna | ostrzega („nie sprawdzono") |

Blokuje tylko to, co da się naprawić: podbicie wersji. Podatność bez poprawki to decyzja
(obejście, wymiana pakietu, akceptacja ryzyka) — należy do człowieka, nie do pętli.

**Co wychodzi z maszyny.** Nazwy i wersje pakietów z lockfile'i idą do `api.osv.dev`.
Wyniki są pamiętane w `~/.cache/ralph/` (12 godzin).

Akceptacja ryzyka (podatność nie dotyczy sposobu użycia, poprawka łamie zgodność) → wyjątek
po odcisku w `ralph/KONTROLE_WYJATKI.md`, wpisuje człowiek. Odcisk jest per podatność
i pakiet — nowa podatność w tym samym pakiecie zablokuje od nowa.
