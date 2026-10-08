---
rodzaj: komenda
uruchom: python3 run.py
kiedy: commit, ci
zakres: zmienione
przy_bledzie: blokuje
---

# Kontrola: zależności

Kod pisany przez model ma własną furtkę w łańcuchu dostaw: model potrafi „wymyślić" pakiet
o wiarygodnej nazwie, której w rejestrze nie ma — a taką nazwę może zarejestrować każdy,
także ktoś, kto na to czeka (*slopsquatting*). Do tego literówki w nazwach popularnych
pakietów (*typosquatting*) i pakiety sprzed kilku dni.

**Co sprawdza.** Każdy pakiet, który w tym commicie **pojawił się** w manifeście
(`package.json`, `requirements*.txt`, `pyproject.toml`, `deno.json`) — porównanie z wersją
z HEAD, więc stare zależności nie są sprawdzane przy każdym commicie.

| Sygnał | Waga |
|---|---|
| pakietu nie ma w rejestrze (npm, PyPI, JSR) | blokuje |
| pakiet młodszy niż 7 dni | blokuje |
| nazwa o 1–2 znaki od popularnego pakietu **i** pakiet młody albo mało pobierany | blokuje |
| nazwa podobna do popularnego pakietu | ostrzega |
| pakiet młodszy niż 30 dni, mniej niż 50 pobrań tygodniowo (npm), oznaczony jako przestarzały | ostrzega |
| skrypty instalacyjne (`preinstall`/`install`/`postinstall`) — kod uruchamiany przy `npm install` | ostrzega |
| zależność spoza rejestru (git, URL) — omija rejestr i jego historię wersji | ostrzega |
| dodatkowy indeks pakietów (`--extra-index-url`, `registry=` w `.npmrc`) — klasyczna droga *dependency confusion* | ostrzega |
| rejestr nieosiągalny | ostrzega („nie sprawdzono") — praca offline nie staje |

**Co wychodzi z maszyny.** Nazwy nowych pakietów idą do publicznych rejestrów — tak samo jak
przy instalacji. Pakiety ze scope'ów, dla których `.npmrc` wskazuje prywatny rejestr
(`@firma:registry=…`), są pomijane. Wyniki są pamiętane w `~/.cache/ralph/` (doba).

Pakiet, który jest w porządku mimo sygnału (świeży, ale własny; podobna nazwa celowo) →
wyjątek po odcisku w `ralph/KONTROLE_WYJATKI.md`, wpisuje człowiek.
