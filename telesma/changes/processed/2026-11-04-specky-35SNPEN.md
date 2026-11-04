---
date: 2026-11-04
kind: feature
affects: general (nowe wymaganie Specky VA-CONF-2; nowy crate va-system-audio, Info.plist, messages.rs)
specky_req: 01M6JR8RPT9TZGC0B8F35SNPEN
specky_hash: c3973d5
specky_reason: what_changed change_proposal (rozstrzygnięte, seq 948)
---
# VA-CONF-2 — przechwytywanie dźwięku systemowego (Core Audio process tap)

Dźwięk systemowy z Core Audio process tap (macOS 14.2+), bez wirtualnych urządzeń i bez zmiany wyjścia. Zgoda „Nagrywanie dźwięku systemowego” przy pierwszym użyciu; odmowa → nagranie spotkania nie startuje, powiadomienie, gdzie włączyć zgodę. Decyzja właściciela 2026-11-04: implementacja w Ruście, `unsafe` tylko w crate'cie va-system-audio.

Kryteria: NSAudioCaptureUsageDescription w Info.plist i pytanie systemu przy pierwszym użyciu; odmowa → szara ikona, powiadomienie ze ścieżką w Ustawieniach, zwykłe nagranie działa; afplay speech_en.wav przy wyciszonym mikrofonie → słowa kluczowe w schowku; wyjście i głośność bez zmian, brak nowego urządzenia audio; macOS < 14.2 → pozycja nieaktywna, skrót daje powiadomienie; dźwięk systemowy nie idzie do sieci ani do logów.
