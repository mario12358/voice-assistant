---
data: 2026-10-10
rodzaj: rozwój
priorytet: normalny
dotyczy: ogólne (nowe wymaganie Specky, sekcja „Ikona w pasku menu”; kod: nowy moduł login_item.rs, tray_menu.rs, app.rs)
specky_req: 01M4KD15CZM8DT7A5CRS3WPKBD
specky_hash: 621aaf5
---

# VA-SET-2 — „Uruchamiaj przy logowaniu” (LaunchAgent)

Zaakceptowane 2026-10-10 (wsad 01M4KC9EX5DBY9R1XYWNVHGX4Z), powód z kolejki: `to_build`.

**Treść (v1 @621aaf5):** Aplikacja może uruchamiać się automatycznie przy logowaniu użytkownika;
pozycja „Uruchamiaj przy logowaniu” w menu ikony pokazuje i przełącza ten stan.

**Kryteria:**
1. (crit: 01M4KD15DB7W591JT9P5FYP7Q9) Pozycja zaznaczona dokładnie wtedy, gdy istnieje plik LaunchAgent w `~/Library/LaunchAgents` wskazujący na tę aplikację
2. (crit: 01M4KD15DB3BM58PTSR1008YN8) Włączenie tworzy wpis wskazujący na bieżący bundle, wyłączenie usuwa; zmiana od razu w menu
3. (crit: 01M4KD15DBKW3YA6G87T6VKEJR) Po wylogowaniu i zalogowaniu aplikacja startuje sama (test ręczny)
4. (crit: 01M4KD15DBAF4A3ATNKW62GVV1) Pozycja nieaktywna poza bundlem `.app` (np. `cargo run`), z podpowiedzią dlaczego

**Kontekst:** LaunchAgent (plik plist z `RunAtLoad`) zamiast SMAppService — bez `unsafe`
i bez nowych zależności; ścieżka bundla z `std::env::current_exe()` (`…/VoiceAsystent.app/Contents/MacOS/VoiceAsystent`).
