#!/usr/bin/env bash
# Sprawdza obraz instalacyjny przed publikacją (VA-CI-1): rozmiar poniżej limitu i brak pliku
# modelu w środku (model pobiera aplikacja po instalacji — VA-MODEL-1).
#
# Użycie: scripts/check-dmg.sh <obraz.dmg> [--max-mb N]   (domyślnie 20 MB)
# Kod wyjścia: 0 = obraz w porządku, 1 = za duży albo zawiera model, 2 = zły argument.
set -euo pipefail

DMG=""
MAX_MB=20
while [[ $# -gt 0 ]]; do
  case "$1" in
    --max-mb) MAX_MB="$2"; shift 2 ;;
    *) DMG="$1"; shift ;;
  esac
done
if [[ -z "$DMG" || ! -f "$DMG" ]]; then
  echo "brak obrazu: ${DMG:-<nie podano>}" >&2
  exit 2
fi

SIZE=$(stat -f%z "$DMG")
LIMIT=$((MAX_MB * 1000 * 1000))
if (( SIZE >= LIMIT )); then
  echo "obraz za duży: ${SIZE} B (limit ${MAX_MB} MB)" >&2
  exit 1
fi

MOUNT="$(mktemp -d)"
cleanup() {
  hdiutil detach -quiet "$MOUNT" 2>/dev/null || true
  rmdir "$MOUNT" 2>/dev/null || true
}
trap cleanup EXIT
hdiutil attach -quiet -nobrowse -readonly -mountpoint "$MOUNT" "$DMG"

MODELS=$(find "$MOUNT" \( -name '*.bin' -o -iname 'ggml*' \) -not -type l 2>/dev/null || true)
if [[ -n "$MODELS" ]]; then
  echo "obraz zawiera plik modelu:" >&2
  echo "$MODELS" >&2
  exit 1
fi
echo "obraz w porządku: ${SIZE} B, bez modelu"
