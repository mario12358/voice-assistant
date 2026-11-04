---
date: 2026-11-04
kind: feature
affects: general (nowe wymaganie Specky VA-CONF-3; va-config, va-audio, kontroler)
specky_req: 01M6JR8RX0HYBEA57XWREDTSAM
specky_hash: cf71279
specky_reason: what_changed change_proposal (rozstrzygnięte, seq 949)
---
# VA-CONF-3 — długie nagranie spotkania na dysku

Własny limit spotkania `max_meeting_secs` (domyślnie 7200 s), niezależny od `max_recording_secs`. Dźwięk w trakcie dopisywany do pliku tymczasowego w ~/Library/Application Support/VoiceAsystent/ (0600, 16 kHz mono), nie w RAM. Limit → auto-Stop z transkrypcją i powiadomieniem jak VA-REC-6. Plik usuwany po transkrypcji (także po błędzie), pozostałość po awarii usuwana przy starcie.

Kryteria: domyślne 7200 s i osobny limit; auto-Stop z powiadomieniem; plik 16 kHz mono 0600, pamięć < 300 MB ponad stan po załadowaniu modelu po 60 min (ręcznie); usuwanie pliku; 60 min → transkrypcja w schowku i historii, czas zmierzony ręcznie.
