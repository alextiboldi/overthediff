#!/usr/bin/env bash
# Regenerate the Linux hicolor icons from assets/app-icon/overthediff.svg. The outputs are
# committed, so packaging never needs this script's tool (rsvg-convert, from librsvg2-bin).
#
#   packaging/linux/icons.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SVG="$ROOT/assets/app-icon/overthediff.svg"
OUT="$ROOT/assets/app-icon/hicolor"
ID=io.github.alextiboldi.overthediff

command -v rsvg-convert >/dev/null || { echo "error: rsvg-convert not found (apt install librsvg2-bin)" >&2; exit 1; }
for s in 16 24 32 48 64 128 256 512; do
  mkdir -p "$OUT/${s}x${s}/apps"
  rsvg-convert -w "$s" -h "$s" -o "$OUT/${s}x${s}/apps/$ID.png" "$SVG"
done
mkdir -p "$OUT/scalable/apps"
cp "$SVG" "$OUT/scalable/apps/$ID.svg"
echo "icons written to $OUT"
