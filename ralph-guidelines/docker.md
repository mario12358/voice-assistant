---
tag: docker
keywords: docker, compose, kontener, dockerfile, obraz, deploy, sieć, sekrety, wolumen
---

# Wytyczne: Docker / Compose

## Topologia

- Usługi w prywatnej sieci kontenerowej; z internetu wystawiona WYŁĄCZNIE warstwa wejściowa HTTP/HTTPS
- Port PostgreSQL NIE jest publikowany na publicznym interfejsie (żadnego `ports:` dla bazy; dostęp administracyjny przez SSH/tunel)
- Konfiguracja infrastruktury (compose, Caddyfile itd.) wersjonowana w repozytorium

## Obrazy

- Obrazy budowane w CI, wersjonowane jawnym tagiem (SHA/wersja) — nigdy sam `latest`
- Serwer NIE buduje aplikacji — na serwer trafiają gotowe artefakty
- Multi-stage Dockerfile; do obrazu wchodzi tylko to, co potrzebne w runtime
- Proces w kontenerze jako nie-root

## Compose

- Healthcheck dla każdej usługi; zależności przez `depends_on: condition: service_healthy`
- Dane PostgreSQL na nazwanym wolumenie
- `restart: unless-stopped` dla usług stałych
- Limity/przydział zasobów (`cpuset`, limity pamięci) jako jawna konfiguracja — podział CPU baza/aplikacja jest przedmiotem pomiaru eksperymentu, więc musi być sterowalny z compose

## Sekrety i konfiguracja

- Sekrety NIGDY w repo jako jawny tekst i NIGDY zapieczone w obrazie
- Konfiguracja przez zmienne środowiskowe / pliki env poza repo (`.env` w `.gitignore`, `.env.example` w repo)
- Te same obrazy + inna konfiguracja = inne środowisko

## Deploy i migracje

- Migracje bazy jako JAWNY etap wdrożenia (osobna komenda/task), nie automat przy starcie kontenera
- Deploy uruchamiany ręcznie jest OK przy jednym środowisku — ale powtarzalny (skrypt/task, nie sekwencja z pamięci)
- Rollback = powrót do poprzedniego tagu obrazu; migracje muszą to umożliwiać (zgodność N-1)

## Czego unikać

- `ports:` na bazie i usługach wewnętrznych
- Budowania obrazów na serwerze docelowym
- `latest` jako jedynego tagu wdrożeniowego
- Sekretów w `docker-compose.yml`, Dockerfile lub ARG/ENV commitowanych do repo
- Kontenerów jako root bez powodu
