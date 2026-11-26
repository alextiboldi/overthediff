#!/usr/bin/env bash
# Repackage package.sh's tarball as a single-file Flatpak bundle, built but not a supported
# Release 1 package (E01-S2 Decisions):
#
#   $DIST/overthediff-<version>-linux-<arch>.flatpak
#
# Usage: packaging/linux/flatpak-bundle.sh TARBALL
#
# The shared release workflow's Flatpak job runs it. Needs flatpak and flatpak-builder; the
# freedesktop runtime and SDK the manifest names are installed per user from Flathub. The bundle
# is then installed per user and checked inside its sandbox.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
APP_ID=io.github.alextiboldi.overthediff
FLATHUB=https://dl.flathub.org/repo/flathub.flatpakrepo
TARBALL="${1:?usage: packaging/linux/flatpak-bundle.sh <overthediff-<version>-linux-<arch>.tar.gz>}"
[[ -f "$TARBALL" ]] || { echo "error: $TARBALL not found (run packaging/linux/package.sh first)" >&2; exit 1; }
DIST="${DIST:-$ROOT/dist/release}"
mkdir -p "$DIST"
WORK="${CARGO_TARGET_DIR:-$ROOT/target}/flatpak-bundle"
rm -rf "$WORK"
mkdir -p "$WORK/stage"
tar -xzf "$TARBALL" -C "$WORK/stage" --strip-components=1
cp "$ROOT/packaging/linux/$APP_ID.yml" "$WORK/"

flatpak remote-add --user --if-not-exists flathub "$FLATHUB"
# --disable-rofiles-fuse: containers and CI runners have no FUSE.
flatpak-builder --user --install-deps-from=flathub --disable-rofiles-fuse --force-clean \
  --default-branch=stable --repo="$WORK/repo" --state-dir="$WORK/state" "$WORK/build" "$WORK/$APP_ID.yml"
OUT="$DIST/$(basename "$TARBALL" .tar.gz).flatpak"
flatpak build-bundle --runtime-repo="$FLATHUB" "$WORK/repo" "$OUT" "$APP_ID" stable
echo "wrote $OUT"

flatpak install --user -y --noninteractive --reinstall "$OUT"
flatpak run --command=sh "$APP_ID" -c "test -x /app/bin/otd && test -f /app/share/applications/$APP_ID.desktop && test -f /app/share/icons/hicolor/256x256/apps/$APP_ID.png && test -f /app/share/doc/overthediff/NOTICE"
echo "==> bundle checked"
