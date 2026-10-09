#!/usr/bin/env bash
# Buduje obraz instalacyjny dist/VoiceAsystent-<wersja>.dmg: VoiceAsystent.app i skrót
# do /Applications do przeciągnięcia (VA-PLAT-2). Modelu Whisper w obrazie nie ma —
# aplikacja pobiera go przy pierwszym uruchomieniu (VA-MODEL-1).
#
# Użycie: scripts/build-dmg.sh [--app <VoiceAsystent.app>] [--out <katalog>] [--version <v1.2.3[-sufiks]>]
#   --app      gotowy bundle zamiast budowania go przez scripts/build-app.sh (testy, CI)
#   --out      katalog docelowy (domyślnie dist/)
#   --version  wersja w nazwie pliku (domyślnie `git describe --tags --match 'v*'`)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="$ROOT/dist"
APP=""
VERSION=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --app) APP="$2"; shift 2 ;;
    --out) OUT="$2"; shift 2 ;;
    --version) VERSION="$2"; shift 2 ;;
    *) echo "nieznany argument: $1" >&2; exit 2 ;;
  esac
done

if [[ -z "$VERSION" ]]; then
  VERSION="$(git -C "$ROOT" describe --tags --match 'v*' --always --dirty 2>/dev/null || echo 0.0.0)"
fi

if [[ -z "$APP" ]]; then
  "$ROOT/scripts/build-app.sh" --out "$OUT" --version "$VERSION"
  APP="$OUT/VoiceAsystent.app"
fi
if [[ ! -f "$APP/Contents/Info.plist" ]]; then
  echo "to nie jest bundle aplikacji: $APP" >&2
  exit 1
fi

STAGING="$(mktemp -d)"
trap 'rm -rf "$STAGING"' EXIT
cp -R "$APP" "$STAGING/VoiceAsystent.app"
ln -s /Applications "$STAGING/Applications"

mkdir -p "$OUT"
DMG="$OUT/VoiceAsystent-${VERSION#v}.dmg"
rm -f "$DMG"
hdiutil create -quiet -volname "VoiceAsystent" -srcfolder "$STAGING" -format UDZO -ov "$DMG"
echo "zbudowano: $DMG"
