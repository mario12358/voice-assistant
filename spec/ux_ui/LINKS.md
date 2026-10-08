<!-- RALPH_UX_LINKS_TEMPLATE — usuń tę linię po dodaniu pierwszego prawdziwego wpisu -->
# Rejestr linków do mockupów

Jedno miejsce z całym interfejsem projektowanym poza repo — na start canvasy z Claude Design,
format działa też dla innych narzędzi (Figma itd.). Mockupy plikowe w `spec/ux_ui/` działają
jak dotychczas; oba źródła mogą współistnieć w jednym projekcie.

<!--
Zasady:
- Jeden wpis = jeden mockup/canvas. Nagłówek: `## <id> — Tytuł`.
  `<id>` małe litery/cyfry/myślniki, STABILNY — zadania w PLAN.md referują go tagiem:
  (ui-screen: spec/ux_ui/LINKS.md#<id>)
- Pole `- link:` jest obowiązkowe. Link musi być UDOSTĘPNIONY (publiczny / share link) —
  prywatnego artefaktu Ralph nie pobierze WebFetchem. ralph-start.sh sprawdza osiągalność
  linków przy starcie (tylko ostrzeżenie).
- `- typ:` jeden z: screen | flow | tokens | komponenty
- `- zakres:` co mockup pokazuje (ekrany, stany) — pomaga w indeksowaniu i tagowaniu zadań
- Ten plik NIE podlega zamrożeniu spec/ — możesz dopisywać wpisy w trakcie projektu.
  ralph-start.sh wykryje niezaindeksowane wpisy i Ralph odświeży sekcję ## Wytyczne UX/UI
  w ralph/SPEC_INDEX.md na starcie sesji.

Przykład wpisu (wcięcie celowe — usuń wcięcie w prawdziwym wpisie):

  ## b1-project-overview — Przegląd projektu
  - link: https://claude.ai/public/artifacts/00000000-0000-0000-0000-000000000000
  - typ: screen
  - zakres: lista projektów, karty z metrykami; stany: pusty / załadowany / błąd
-->
