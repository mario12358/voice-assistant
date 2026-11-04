---
date: 2026-11-04
kind: feature
affects: general (nowe wymaganie Specky VA-CONF-4, sekcja „Nagrywanie spotkań”)
specky_req: 01M6JR8S20R2A5MVXYDD6FG0MQ
specky_hash: 403da01
specky_reason: what_changed change_proposal (rozstrzygnięte, seq 950)
---
# VA-CONF-4 — etykiety mówców w transkrypcji spotkania

Ścieżka mikrofonu i ścieżka dźwięku systemowego transkrybowane osobno, scalane chronologicznie po znacznikach czasu; zmiana mówcy oznaczona „Ja:” / „Rozmówcy:”. `meeting_speaker_labels = false` w config.toml → jeden tekst w kolejności czasowej bez etykiet.

Kryteria: (1) speech_pl od 0 s + speech_en od 10 s → „Ja:” potem „Rozmówcy:”, odwrotnie przy odwrotnych przesunięciach; (2) etykieta tylko przy zmianie mówcy, wypowiedzi w nowych liniach; (3) ścieżka samej ciszy nie daje etykiety; (4) skracanie pauz i filtry STT na każdej ścieżce osobno, znaczniki czasu w oryginalnej osi nagrania; (5) bez etykiet przy `false`, zwykłe nagranie nigdy bez etykiet.
