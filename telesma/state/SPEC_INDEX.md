# telesma/state/SPEC_INDEX.md — Indeks dokumentacji

## Statistics

- **Liczba plików**: 3 (2 w `telesma/spec/`, 1 w `telesma/spec/ux_ui/`)
- **Total size**: 215 lines (~16 KB)
- **Indexed on**: 2026-10-08
- **Uwaga**: 2 z 3 plików to niewypełnione szablony (marker `RALPH_APP_FLOW_TEMPLATE`, `RALPH_UX_LINKS_TEMPLATE`). Jedyna treść merytoryczna: 3 wpisy w `WYTYCZNE_TECHNICZNE.md` (Rust, model large-v3-turbo, tylko GPU).

---

### WYTYCZNE_TECHNICZNE.md (86 lines)
Szablon wytycznych technicznych z trzema wypełnionymi polami: język Rust, narzucony model `large-v3-turbo` (Whisper — rozpoznawanie mowy), ograniczenie „model działa tylko na GPU". Pozostałe sekcje (hosting, integracje, NFR) puste.

#### Sections:
| Lines | Section | Anchor | Description |
|-------|--------|---------|------|
| 1-17 | Wprowadzenie / instrukcja | # Wytyczne techniczne projektu | Instrukcja wypełniania szablonu; puste pole = default z guidelines |
| 20-30 | Stack — odstępstwa | ## Stack — odstępstwa od defaultów | Rust jako język (l.24), model large-v3-turbo (l.30); reszta pusta |
| 32-38 | Hosting i infrastruktura | ## Hosting i infrastruktura | Puste: środowisko, kontenery, CI/CD, sekrety, monitoring |
| 40-50 | Integracje zewnętrzne | ## Integracje zewnętrzne | Pusty szablon systemu zewnętrznego `[nazwa]` |
| 52-64 | Wymagania niefunkcjonalne | ## Wymagania niefunkcjonalne (gdy odbiegają od defaultów) | Puste: wydajność, i18n, offline, compliance, a11y |
| 66-70 | Inne ograniczenia | ## Inne ograniczenia | Jedyne ograniczenie: model ma działać tylko na GPU (l.70) |
| 74-86 | Wymagania biznesowe (odsyłacz) | ## Wymagania biznesowe i funkcjonalne | Informacja, że wymagania biznesowe mają być w osobnych plikach — brak takich plików |

#### Keywords: Rust, large-v3-turbo, Whisper, STT, GPU, stack, ograniczenia

---

### APP_FLOW.md (101 lines)
**Niewypełniony szablon** (marker `RALPH_APP_FLOW_TEMPLATE` w l.1). Zawiera przykładowe, generyczne flow aplikacji webowej (rejestracja, tworzenie projektu) niezwiązane z asystentem głosowym. Nie jest traktowany jako źródło wymagań.

#### Sections:
| Lines | Section | Anchor | Description |
|-------|--------|---------|------|
| 1-14 | Marker + wprowadzenie | <!-- RALPH_APP_FLOW_TEMPLATE: usuń tę linię gdy zaczniesz wypełniać szablon --> | Cel pliku: Kroki = scenariusze e2e + nagrania |
| 16-24 | Zasady wypełniania | ## Zasady wypełniania | Reguły pisania Kroków z perspektywy usera |
| 28-32 | Persony (przykład) | ## Persony (przykład — dostosuj do projektu lub usuń) | Przykładowe role Admin/User/Guest |
| 36-54 | Krok 1 (przykład) | ## Step 1 — Rejestracja i pierwsze logowanie | Przykładowy flow rejestracji web — szablonowy |
| 58-81 | Krok 2 (przykład) | ## Step 2 — Utworzenie pierwszego obiektu (projekt / dokument / cokolwiek) | Przykładowy flow tworzenia projektu — szablonowy |
| 85-91 | Krok 3 (placeholder) | ## Step 3 — [Tytuł kolejnego procesu] | Pusty placeholder |
| 95-101 | Notatki dla Ralpha | ## Notatki dla Ralpha (czytaj jako AI) | Mapowanie Kroków na testy e2e i nagrania |

#### Keywords: szablon, flow, e2e, nagranie, persona, krok

---

## UX/UI guidelines

### telesma/spec/ux_ui/LINKS.md (28 lines)
**Type**: rejestr linków — **niewypełniony szablon** (marker `RALPH_UX_LINKS_TEMPLATE` w l.1). Brak wpisów `## <id> — Tytuł`; jedyny przykład jest w komentarzu HTML.

#### Links:
| Id | Type | Link | Description |
|----|-----|------|------|
| — | — | — | Brak wpisów — rejestr pusty |

Brak plików mockupów (`.css`, `.jsx`, `.html`, obrazów) w `telesma/spec/ux_ui/`.

---

## Flows

Nie wystawiono: `Nagrania: nie` w `telesma/config.md`, a `telesma/spec/APP_FLOW.md` ma marker szablonu `RALPH_APP_FLOW_TEMPLATE`.

---

## Documentation quality

- **Completeness**: ~5%
  - Wymagania funkcjonalne: brak (nie wiadomo, co asystent ma robić poza implikacją „rozpoznawanie mowy")
  - Wymagania niefunkcjonalne: tylko „GPU only"; brak latencji, języków, offline/online, prywatności
  - Integracje zewnętrzne: brak (LLM? TTS? system operacyjny? aplikacje sterowane głosem?)
  - Model danych: brak
  - Kryteria akceptacji: brak
  - Obsługa błędów: brak (np. brak GPU, brak mikrofonu)
  - Deployment: brak (platforma docelowa, OS, dystrybucja, pobieranie modelu)
  - Testy: brak
- **Unambiguity**: low — „large-v3-turbo" wskazuje na Whisper, ale nie określa runtime'u (whisper.cpp / candle / CTranslate2); „asystent" może oznaczać dyktowanie, komendy lub rozmowę z LLM
- **Consistency**: high (za mało treści, by powstały sprzeczności; szablonowy APP_FLOW opisuje generyczną aplikację web, ale jest jawnie oznaczony jako szablon)
- **Project size**: nieznana — szacunkowo mały/średni (pipeline audio → STT → [logika] → [wyjście]); zależy od odpowiedzi na pytania o zakres
- **Luki tematyczne**:
  1. Cel produktu i use case'y (dyktowanie? komendy systemowe? konwersacja?)
  2. Platforma docelowa (OS, desktop/serwer, GPU vendor: CUDA / Metal / Vulkan)
  3. Aktywacja nasłuchu (wake word / push-to-talk / ciągły)
  4. Przetwarzanie po transkrypcji (LLM lokalny/chmurowy, akcje)
  5. Wyjście (tekst, wpisywanie do okna, TTS)
  6. Interfejs użytkownika (CLI, tray, GUI, API)
  7. Języki rozpoznawania
  8. Wydajność / latencja / streaming
  9. Prywatność i przechowywanie nagrań/transkrypcji
  10. Dystrybucja modelu i aplikacji, konfiguracja
  11. Obsługa błędów sprzętowych (brak GPU, VRAM, mikrofon)
  12. Strategia testów (fixtures audio, testy na GPU w CI)
- **Rekomendacja**: Spec NIE wystarcza do zaplanowania projektu — przed generowaniem planu konieczne jest doprecyzowanie celu i zakresu (pytania Q1–Q8 krytyczne); idealnie dopisać krótki plik `telesma/spec/` z opisem use case'ów.

<!-- TELESMA_SPEC_CHECKSUMS (generated by telesma.sh — do not edit)
1250110431 5365 telesma/spec/APP_FLOW.md
3122227807 4063 telesma/spec/WYTYCZNE_TECHNICZNE.md
-->
