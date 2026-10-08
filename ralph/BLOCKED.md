RALPH BLOCKED

Zadanie: kontrakt CI ze Specky (sekcja 18.5) — wykryte przy 3.1 (PR #5, zmergowany)
Problem: Specky nie łączy testów Rusta ze znacznikiem `// specky: crit <id>` z wynikami JUnit. Na PR #5 kryterium VA-STT-2 K2 (01M4EKHCMW0ZW9K5GYM7D2NM9H) ma „⚠️ brak testu”, a w Specky `has_test: false`, choć dwa testy mają znacznik i przechodzą w CI (artefakt `junit-reports` jest wgrywany). Bez tego żadne kryterium nie dostanie ✅ z CI.
Próby naprawy: (1) znacznik nad `#[test]` — „brak testu”; (2) znacznik między `#[test]` a `fn` — bez zmian. Podejrzenie: JUnit z cargo-nextest ma `classname` = nazwa crate'u (np. `va-stt`), bez ścieżki pliku, więc Specky nie znajduje pliku źródłowego ze znacznikiem; dokumentacja Specky podaje przykłady tylko dla Pythona i TS.
Wpływ: nie blokuje kodu (checki PR-ów są zielone), ale dowody kryteriów w Specky nie powstają. Zadanie 3.2 i tak czeka na decyzję VA-MODEL-1.

Potrzebuję pomocy z: czy Specky obsługuje znaczniki w Ruście (i w jakim miejscu/formacie), czy mam dopisać w CI krok, który dokleja do raportu JUnit atrybut `file` ze ścieżką testu, czy na razie pracować dalej bez dowodów z CI i sprawdzać kryteria ręcznie na liście w komentarzu PR?
