#!/usr/bin/env bash
# Build OverTheDiff for Linux on this machine's architecture (x86_64 or aarch64) and package it:
#
#   $DIST/overthediff-<version>-linux-<arch>.deb     Debian, Ubuntu and their derivatives
#   $DIST/overthediff-<version>-linux-<arch>.tar.gz  the same tree (bin/, share/), which the shared
#                                                    release workflow's Flatpak job repackages
#
# Usage: packaging/linux/package.sh [--skip-build]
#
# Needs cargo and nfpm (https://nfpm.goreleaser.com). Build on the oldest supported glibc (CI:
# Ubuntu 22.04, glibc 2.35) so the binary runs on newer ones. desktop-file-validate is used when
# it is installed. Only x86_64 is a supported Release 1 package (E01-S2 Decisions); the shared
# workflow also runs this on aarch64, which builds the same way but is not supported.
#
# Environment: OTD_VERSION overrides the [workspace.package] version, DIST the output folder
# (default dist/release), CARGO_TARGET_DIR cargo's (default target).
set -euo pipefail
# shellcheck source=../env.sh
. "$(dirname "${BASH_SOURCE[0]}")/../env.sh"
HERE="$ROOT/packaging/linux"
APP_ID=io.github.alextiboldi.overthediff

SKIP_BUILD=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --skip-build) SKIP_BUILD=1; shift ;;
    -h | --help) sed -n '2,16p' "$0"; exit 0 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done

DIST="$(cd "$DIST" && pwd)"

ARCH="$(uname -m)"
case "$ARCH" in
  x86_64) DEB_ARCH=amd64 ;;
  aarch64 | arm64) ARCH=aarch64; DEB_ARCH=arm64 ;;
  *) echo "error: unsupported architecture $ARCH" >&2; exit 2 ;;
esac
BASENAME="overthediff-$VERSION-linux-$ARCH"
echo "==> OverTheDiff $VERSION for Linux $ARCH"

if [[ "$SKIP_BUILD" = 0 ]]; then
  (cd "$ROOT" && cargo build --release --locked -p otd)
fi
WORK="$CARGO_TARGET_DIR/linux-package"
STAGE="$WORK/root"
rm -rf "$WORK"

# The installed tree, shared by both formats.
install -Dm755 "$CARGO_TARGET_DIR/release/otd" "$STAGE/usr/bin/otd"
strip "$STAGE/usr/bin/otd" 2>/dev/null || true
install -Dm644 "$HERE/$APP_ID.desktop" "$STAGE/usr/share/applications/$APP_ID.desktop"
mkdir -p "$STAGE/usr/share/icons"
cp -R "$ROOT/assets/app-icon/hicolor" "$STAGE/usr/share/icons/"
# The notices every package carries (E01-S2). The repository has no licence file of its own yet.
mkdir -p "$STAGE/usr/share/doc/overthediff"
copy_notices "$STAGE/usr/share/doc/overthediff"

# The launcher finds the entry only if its icon and executable are where it names them.
icon="$(sed -n 's/^Icon=//p' "$STAGE/usr/share/applications/$APP_ID.desktop")"
exe="$(sed -n 's/^Exec=\([^ ]*\).*/\1/p' "$STAGE/usr/share/applications/$APP_ID.desktop")"
[[ -f "$STAGE/usr/share/icons/hicolor/256x256/apps/$icon.png" ]] || { echo "error: no 256x256 icon named $icon" >&2; exit 1; }
[[ -x "$STAGE/usr/bin/$exe" ]] || { echo "error: the desktop entry starts $exe, which is not in usr/bin" >&2; exit 1; }
if command -v desktop-file-validate >/dev/null; then
  desktop-file-validate "$STAGE/usr/share/applications/$APP_ID.desktop"
fi

mkdir -p "$WORK/tar"
cp -R "$STAGE/usr" "$WORK/tar/$BASENAME"
tar -C "$WORK/tar" -czf "$DIST/$BASENAME.tar.gz" "$BASENAME"
echo "wrote $DIST/$BASENAME.tar.gz"

command -v nfpm >/dev/null || { echo "error: nfpm not found (https://nfpm.goreleaser.com/install/)" >&2; exit 1; }
export NFPM_ARCH="$DEB_ARCH"
sed "s|\${STAGE}|$STAGE|g" "$HERE/nfpm.yaml" >"$WORK/nfpm.yaml"
(cd "$ROOT" && nfpm package -f "$WORK/nfpm.yaml" -p deb -t "$DIST/$BASENAME.deb")
echo "==> done"
ls -lh "$DIST"
