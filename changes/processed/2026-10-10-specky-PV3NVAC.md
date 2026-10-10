---
data: 2026-10-10
rodzaj: rozwój
priorytet: normalny
dotyczy: ogólne (nowe wymaganie Specky, sekcja „Ikona w pasku menu”; kod: crates/core/src/logging.rs, tray_menu.rs, app.rs, main.rs)
specky_req: 01M4KD15FGKP1T46QXWPV3NVAC
specky_hash: bb6317e
---

# VA-OPS-1 — „Pokaż logi” w menu i retencja logów 7 dni

Zaakceptowane 2026-10-10 (wsad 01M4KC9EX5DBY9R1XYWNVHGX4Z), powód z kolejki: `to_build`.

**Treść (v1 @bb6317e):** Logi aplikacji są łatwo dostępne i nie rosną bez końca: pozycja „Pokaż
logi” w menu otwiera katalog logów w Finderze, a pliki starsze niż 7 dni są usuwane przy starcie.

**Kryteria:**
1. (crit: 01M4KD15FXQ488KDR36QPHD62D) „Pokaż logi” otwiera `~/Library/Logs/VoiceAsystent` w Finderze
2. (crit: 01M4KD15FXMVKSVBYXV6QS4ED6) Przy starcie usuwane `voice-asystent.log.*` starsze niż 7 dni, z logiem ile usunięto
3. (crit: 01M4KD15FXMYC7NHSDBX6BGCTJ) Inne pliki w katalogu nieruszane

**Kontekst:** `tracing_appender::rolling::daily` tworzy plik dziennie i nigdy nie czyści.
