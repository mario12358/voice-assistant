TELESMA BLOCKED

Task: start sesji 2026-11-25 — commit plików zaktualizowanych przez launcher (kontrole, hook uprawnień, config, rotacja REPORT.md); bez niego `next` nie wydaje zadania (brudne drzewo)
Problem: poprzednia blokada (pole Linter/formatter) rozwiązana — lint i testy przechodzą. Teraz kontrola integralności zatrzymuje commit: „the check files differ from what the human approved: telesma/config.md -> ## Checks (changed)”. To nie jest błąd w kodzie — sekcję `## Checks` musi przejrzeć i zatwierdzić człowiek.
Fix attempts: jedna próba `commit --kind artifacts` po poprawce pola Linter/formatter; zatwierdzenia nie wykonuję sam.
Impact: blokuje wszystko (`next` nie wydaje zadań przy brudnym drzewie; PR #81 z 10.Z1.1 / VA-UI-3 czeka na rekoncyliację).
Related history: blokada z 2026-11-25 (pole Linter/formatter) — rozwiązana przez właściciela.

Need help with: przejrzyj różnicę w `telesma/config.md → ## Checks` (`git diff telesma/config.md`) i zatwierdź w terminalu:
`python3 telesma/checks/telesma-checks.py --approve`
Potem uruchom Telesmę ponownie.
