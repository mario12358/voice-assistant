#!/usr/bin/env bash
# Buduje dist/VoiceAsystent.app: release build (Metal przez va-stt), bundle z Info.plist
# i ikoną, podpis ad-hoc (bez notaryzacji). Model Whisper NIE wchodzi do bundla —
# aplikacja pobiera go przy pierwszym uruchomieniu (VA-MODEL-1).
#
# Użycie: scripts/build-app.sh [--out <katalog>] [--binary <plik>] [--version <v1.2.3[-sufiks]>]
#   --out      katalog docelowy (domyślnie dist/)
#   --binary   gotowa binarka zamiast `cargo build --release` (testy, CI)
#   --version  wersja do Info.plist (domyślnie `git describe --tags --match 'v*'`)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="$ROOT/dist"
BINARY=""
VERSION=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --out) OUT="$2"; shift 2 ;;
    --binary) BINARY="$2"; shift 2 ;;
    --version) VERSION="$2"; shift 2 ;;
    *) echo "nieznany argument: $1" >&2; exit 2 ;;
  esac
done

if [[ -z "$BINARY" ]]; then
  cargo build --release -p voice-asystent --manifest-path "$ROOT/Cargo.toml"
  BINARY="$ROOT/target/release/voice-asystent"
fi
if [[ ! -x "$BINARY" ]]; then
  echo "brak wykonywalnej binarki: $BINARY" >&2
  exit 1
fi

if [[ -z "$VERSION" ]]; then
  VERSION="$(git -C "$ROOT" describe --tags --match 'v*' --always --dirty 2>/dev/null || echo 0.0.0)"
fi
SHORT_VERSION="${VERSION#v}"
SHORT_VERSION="${SHORT_VERSION%%-*}"
if [[ ! "$SHORT_VERSION" =~ ^[0-9]+(\.[0-9]+){0,2}$ ]]; then
  SHORT_VERSION="0.0.0"
fi

APP="$OUT/VoiceAsystent.app"
CONTENTS="$APP/Contents"
rm -rf "$APP"
mkdir -p "$CONTENTS/MacOS" "$CONTENTS/Resources"
cp "$BINARY" "$CONTENTS/MacOS/VoiceAsystent"
printf 'APPL????' > "$CONTENTS/PkgInfo"

ICON_WORK="$(mktemp -d)"
trap 'rm -rf "$ICON_WORK"' EXIT
python3 "$ROOT/scripts/app-icon.py" "$ICON_WORK/AppIcon.iconset"
iconutil -c icns "$ICON_WORK/AppIcon.iconset" -o "$CONTENTS/Resources/AppIcon.icns"

cat > "$CONTENTS/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>CFBundleDevelopmentRegion</key>
	<string>pl</string>
	<key>CFBundleDisplayName</key>
	<string>VoiceAsystent</string>
	<key>CFBundleExecutable</key>
	<string>VoiceAsystent</string>
	<key>CFBundleIconFile</key>
	<string>AppIcon</string>
	<key>CFBundleIdentifier</key>
	<string>io.github.mario12358.voiceasystent</string>
	<key>CFBundleInfoDictionaryVersion</key>
	<string>6.0</string>
	<key>CFBundleName</key>
	<string>VoiceAsystent</string>
	<key>CFBundlePackageType</key>
	<string>APPL</string>
	<key>CFBundleShortVersionString</key>
	<string>$SHORT_VERSION</string>
	<key>CFBundleVersion</key>
	<string>$VERSION</string>
	<key>LSApplicationCategoryType</key>
	<string>public.app-category.productivity</string>
	<key>LSMinimumSystemVersion</key>
	<string>13.0</string>
	<key>LSUIElement</key>
	<true/>
	<key>NSHighResolutionCapable</key>
	<true/>
	<key>NSMicrophoneUsageDescription</key>
	<string>VoiceAsystent nagrywa Twój głos z mikrofonu, aby zamienić go na tekst i skopiować do schowka. Nagranie nie opuszcza tego komputera.</string>
</dict>
</plist>
PLIST

plutil -lint "$CONTENTS/Info.plist"
codesign --force --sign - "$APP"
codesign --verify --strict "$APP"
echo "zbudowano: $APP (wersja $VERSION)"
