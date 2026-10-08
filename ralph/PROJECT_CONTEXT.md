# PROJECT_CONTEXT.md

Ten plik przechowuje kontekst projektu który przetrwa `/compact`.

## Stack technologiczny

- Język: Rust (workspace Cargo)
- Platforma: wyłącznie macOS (Apple Silicon), dystrybucja .dmg
- STT: whisper-rs + Whisper large-v3-turbo (GGML), backend Metal
- UI: aplikacja paska menu (tray-icon + muda + tao), skróty global-hotkey, schowek arboard
- Testy: cargo test (testy z modelem/GPU/mikrofonem `#[ignore]`)

## Architektura

[Krótki opis struktury projektu]

## Kluczowe decyzje

<!-- Tabela rośnie z każdą sesją, a plik trafia w całości do promptu startowego, więc         -->
<!-- ralph-start.sh trzyma w niej 40 ostatnich wierszy — starsze idą do ralph/CONTEXT_ARCHIVE.md -->
<!-- (write-only). Decyzję wciąż OBOWIĄZUJĄCĄ — taką, której złamanie zepsułoby projekt —     -->
<!-- oznacz 📌 w kolumnie Decyzja: wiersz z 📌 nie podlega rotacji nigdy.                      -->

| Data | Decyzja | Powód |
|------|---------|-------|
| 2026-10-08 | 📌 Aplikacja wyłącznie na macOS, instalowana z .dmg; Windows/Linux poza zakresem | decyzja właściciela |
| 2026-10-08 | 📌 Specky (projekt 01M4EJNFTHDZ3ECMHT7425APR0) jest jedynym źródłem wymagań; `wymagania.md` to tylko wsad importu | decyzja właściciela |
| 2026-10-08 | 📌 Model rozpoznawania mowy tylko na GPU (Metal), bez fallbacku CPU | spec/WYTYCZNE_TECHNICZNE.md |
| 2026-10-08 | Zakres: nagranie → transkrypcja → schowek; bez LLM, TTS, wpisywania do okna (wcześniejszy plan Linux/CUDA porzucony) | wymagania.md |

## Znane problemy i rozwiązania

<!-- Znane pułapki: destylowane z Logu problemów przy checkpoincie końca fazy (sekcja 7    -->
<!-- instrukcji). Tylko wzorce POWTARZALNE — błąd jednorazowy zostaje w Logu problemów     -->
<!-- w REPORT.md. Jeden wiersz = jedna pułapka, w JEDNEJ linii i zwięźle: plik idzie       -->
<!-- w całości do promptu każdej sesji.                                                     -->
<!-- ralph-start.sh trzyma tu 30 ostatnich wierszy, starsze → ralph/CONTEXT_ARCHIVE.md.    -->
<!-- Pułapkę, która WRACA mimo zapisu (powtórzyła się ≥2 razy), oznacz 📌 — taki wiersz    -->
<!-- nie rotuje się nigdy. Jednorazową wpadkę zostaw bez znacznika.                        -->

| Problem | Rozwiązanie |
|---------|-------------|
| | |

## Zależności między komponentami

[Opis jak komponenty ze sobą współpracują]

## Aktualny stan

<!-- STAN BIEŻĄCY — NADPISUJ. To ma być odpowiedź na „gdzie jesteśmy DZIŚ", a nie kronika  -->
<!-- tur: historia realizacji ma własne miejsce w REPORT.md i tam się rotuje, a tutaj nie  -->
<!-- rotuje się nic. Dopisywanie kolejnych akapitów „tura z 19.08 domknięta" zamienia tę   -->
<!-- sekcję w drugi, nieograniczony raport w prompcie każdej sesji.                        -->

- Ostatnie ukończone zadanie: brak (plan przepisany pod macOS 2026-10-08)
- Następne zadanie: 1.1
- Blokery: brak toolchainu Rust (od 1.2); 11 wymagań czeka na akceptację w Specky (kotwice)
