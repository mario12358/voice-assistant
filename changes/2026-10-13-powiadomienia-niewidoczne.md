---
data: 2026-10-13
rodzaj: błąd
priorytet: krytyczny
dotyczy: VA-UX-1 (01M4KD151ZKD1RNYJNVTGDZ530), VA-REC-6 (01M4K06AXGNSZ9XEZ3E9VFYND0), VA-PLAT-2 (komunikaty 5.5); apps/voice-asystent/src/messages.rs
---
# Powiadomienia systemowe aplikacji nie pojawiają się wcale

Test ręczny właściciela 2026-10-13 (zadania 7.7 i 9.10): po transkrypcji nie ma powiadomienia
„Transkrypcja w schowku”, a przy limicie nagrania nie ma „Osiągnięto limit długości nagrania”.
W Centrum powiadomień też nie ma wpisów. Logi nie zawierają ostrzeżenia
„powiadomienie systemowe nie zostało wysłane”, więc `mac_notification_sys::send_notification`
zwraca sukces, a powiadomienie ginie po stronie systemu (macOS 26.6).

Diagnoza wstępna: `messages.rs` (od zadania 5.5) nie woła `set_application`, więc biblioteka
podszywa się pod `com.apple.Finder` (stare API NSUserNotification + podmiana identyfikatora
bundla). Na aktualnym macOS takie powiadomienia są odrzucane bez błędu.

Oczekiwane:
- wszystkie powiadomienia aplikacji (gotowa transkrypcja, limit nagrania, brak mikrofonu/GPU/modelu)
  są widoczne jako banery i w Centrum powiadomień, podpisane jako VoiceAsystent
  (`io.github.mario12358.voiceasystent`), z osobną pozycją w Ustawieniach systemowych → Powiadomienia;
- przy pierwszym powiadomieniu system pyta o zgodę; gdy użytkownik ją odebrał, aplikacja loguje
  ostrzeżenie (raz), a nie gubi powiadomień po cichu;
- poza bundlem (`cargo run`) zachowanie zastępcze udokumentowane w README/RUNBOOK.

Do sprawdzenia przy realizacji: UNUserNotificationCenter (wymaga bundla z identyfikatorem
i podpisu — ad-hoc z build-app.sh), `set_application` z własnym identyfikatorem, ewentualnie
`osascript display notification` jako zastępstwo. Rozstrzygnięcie zapisz w PROJECT_CONTEXT.
Test ręczny: powiadomienie po transkrypcji i przy limicie (`max_recording_secs = 20`) widoczne.
