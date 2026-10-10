---
data: 2026-10-10
rodzaj: rozwój
priorytet: normalny
dotyczy: ogólne (nowe wymaganie Specky, sekcja „Nagrywanie i schowek”; kod: crates/audio/src/recorder.rs, crates/config, kontroler, messages.rs)
specky_req: 01M4K06AXGNSZ9XEZ3E9VFYND0
specky_hash: 97b27f2
---

# VA-REC-6 — limit długości nagrania z automatycznym Stop i powiadomieniem

Nowe wymaganie zaakceptowane przez właściciela 2026-10-10 (wsad 01M4JZS73RFCBAXE91TSTJCG2V),
powód z kolejki: `to_build` (status `open`).

**Treść (v1 @97b27f2):** Nagranie ma limit długości (domyślnie 10 minut, konfigurowalny). Po
osiągnięciu limitu nagrywanie kończy się samo i nagrany materiał trafia do transkrypcji jak po
ctrl+cmd+s, a użytkownik dostaje powiadomienie, że limit został osiągnięty. Żadna część wypowiedzi
sprzed limitu nie ginie.

**Kryteria:**
1. (crit: 01M4K06AY61DWRJ5R2DFNDJD6W) Domyślny limit nagrania to 600 s; wartość max_recording_secs w config.toml nadal go zmienia
2. (crit: 01M4K06AY60EVTQQ78X8RQ4RCK) Po osiągnięciu limitu nagrywanie zatrzymuje się samo: ikona wraca do szarego, a dźwięk z całego czasu do limitu trafia do transkrypcji i do schowka
3. (crit: 01M4K06AY6B3424GJDBAQQK3WY) W chwili automatycznego zatrzymania użytkownik dostaje powiadomienie systemowe informujące o osiągnięciu limitu długości nagrania
4. (crit: 01M4K06AY6N6SB7WBZE4J4MM09) Nagranie zatrzymane ręcznie przed limitem działa jak dotąd (VA-REC-2)

**Kontekst („było”):** `max_recording_secs` domyślnie 300; po zapełnieniu bufora `append_limited`
tylko ustawia `truncated`, ikona zostaje czerwona, użytkownik nic nie wie, a po ręcznym Stop
transkrybowane jest pierwsze 5 minut z samym ostrzeżeniem w logu. „Jest”: nagrywarka ma zgłosić
osiągnięcie limitu do kontrolera, kontroler wykonuje tę samą ścieżkę co Stop (automat stanów,
Input::Stop), UI dostaje zdarzenie i pokazuje powiadomienie.
