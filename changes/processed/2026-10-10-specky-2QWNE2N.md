---
data: 2026-10-10
rodzaj: rozwój
priorytet: normalny
dotyczy: ogólne (nowe wymaganie Specky, sekcja „Rozpoznawanie mowy”; kod: crates/model/src/lib.rs, startup.rs)
specky_req: 01M4KD155TWYHX07DQC2QWNE2N
specky_hash: 159765b
---

# VA-PERF-1 — szybki start: suma SHA-256 modelu pamiętana po rozmiarze i dacie pliku

Zaakceptowane 2026-10-10 (wsad 01M4KC9EX5DBY9R1XYWNVHGX4Z), powód z kolejki: `to_build`.

**Treść (v1 @159765b):** Aplikacja startuje szybko: suma SHA-256 modelu nie jest liczona przy
każdym uruchomieniu, lecz tylko gdy plik modelu zmienił rozmiar lub datę modyfikacji od ostatniej
udanej weryfikacji; ikona w pasku menu pojawia się bez kilkusekundowego opóźnienia.

**Kryteria:**
1. (crit: 01M4KD15697ZGFE288GDZGAZF6) Po udanej weryfikacji znacznik (rozmiar, data, suma) obok modelu; kolejny start z niezmienionym plikiem pomija liczenie (log)
2. (crit: 01M4KD1569JT903G56F1TTX1VH) Zmiana rozmiaru lub daty wymusza ponowne liczenie; uszkodzony plik nadal wykrywany i usuwany
3. (crit: 01M4KD1569MV3PT2045PB3N6TC) Start → ikona ≤ 1,5 s przy gotowym modelu (pomiar ręczny)
4. (crit: 01M4KD15698XHWP7W8FXSF3DB7) `va-dev model-download` korzysta z tego samego mechanizmu

**Kontekst („było”):** `ModelStore::check` liczy SHA-256 1,6 GB przy każdym starcie (ok. 3,5 s
w wydaniu, ~1 min w buildzie debug) — zadanie 5.1 odnotowało to w logu problemów. Świadomy
kompromis: uszkodzenie bez zmiany rozmiaru i daty nie zostanie wykryte przez znacznik; „Pobierz
ponownie” i pobieranie zawsze liczą sumę od nowa.
