---
rodzaj: komenda
uruchom: python3 run.py
sprawdz: python3 run.py --sprawdz
kiedy: commit, ci
zakres: zmienione
przy_bledzie: blokuje
---

# Kontrola: sekrety

Sekret w repozytorium zostaje w historii gita na zawsze — usunięcie w następnym commicie
go nie cofa, trzeba go unieważnić. Dlatego ta kontrola działa **przed** commitem.

**Co łapie.** Klucze w znanych formatach (AWS, GitHub, GitLab, Slack, OpenAI/Anthropic,
Google, Stripe, JWT, klucze prywatne PEM) i pliki, które z nazwy są sekretami (`.env`,
`*.pem`, `id_rsa`…) — te blokują. Przypisanie hasła/tokenu w kodzie (`password = "…"`) tylko
ostrzega: w testach i fiksturach bywa celowe.

**Narzędzie.** Gdy w PATH jest `gitleaks`, kontrola używa go (kilkaset reguł) i dokłada
własne reguły nazw plików. Bez niego działa tryb wbudowany — kilkanaście reguł z tego pliku.
Walidator przy starcie mówi, który tryb jest aktywny.

**Czego nie robi.** Nigdy nie wypisuje treści sekretu — w logu i w komunikacie dla Claude'a
jest reguła, plik, linia i odcisk (skrót SHA). Nie sprawdza historii gita (tylko pliki
w zakresie punktu).

Fałszywy alarm (np. klucz testowy z dokumentacji dostawcy) → wyjątek w
`ralph/KONTROLE_WYJATKI.md` po odcisku, wpisuje człowiek.
