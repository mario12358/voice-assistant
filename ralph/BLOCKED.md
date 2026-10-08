RALPH BLOCKED

Zadanie: 1.2 i następne (1.1 jest wykonalne)
Problem:
1. Na Macu nie ma toolchainu Rust (`cargo`, `rustc`), więc nie da się zbudować ani przetestować kodu.
2. 11 wymagań czeka w Specky na akceptację (wsad 01M4EK43A6GBPR63H0FV0VBCV0). Bez akceptacji nie ma `requirement_id`, więc nie da się wpisać kotwic do planu ani ustawić statusu `in_progress`.
3. Trzeba zdecydować, jak dostarczyć model (~1,6 GB). Domyślnie: model dołączony do .app, aplikacja działa offline, a .dmg waży ~1,6 GB. Alternatywa: pobranie modelu przy pierwszym uruchomieniu.
Wpływ: blokuje wszystko od zadania 1.2.

Potrzebuję pomocy z: instalacją Rust (`! curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`), akceptacją wsadu w Specky i decyzją w sprawie dostarczenia modelu.
