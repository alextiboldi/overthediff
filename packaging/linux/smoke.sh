#!/usr/bin/env bash
# Smoke test for the .deb: install it, check what the application launcher reads, then start the
# app through its desktop entry and wait for its window.
#
#   packaging/linux/smoke.sh dist/release/overthediff-<version>-linux-<arch>.deb
#
# Run it as root or with sudo on a disposable machine (CI: Ubuntu 22.04 and 24.04), since it
# installs the package system-wide. Needs apt, gio (libglib2.0-bin), xwininfo (x11-utils) and, with
# no display, xvfb-run (xvfb); with no GPU, Mesa's software drivers (mesa-vulkan-drivers). The app
# runs with its settings kept in memory, so it writes nothing to the user's profile.
set -euo pipefail
DEB="${1:?usage: packaging/linux/smoke.sh <package.deb>}"
APP_ID=io.github.alextiboldi.overthediff
ENTRY="/usr/share/applications/$APP_ID.desktop"
SUDO=
[[ "$(id -u)" = 0 ]] || SUDO=sudo

fail() { echo "smoke: FAIL: $*" >&2; exit 1; }

echo "==> install $DEB"
$SUDO apt-get install -y "$(cd "$(dirname "$DEB")" && pwd)/$(basename "$DEB")"

echo "==> check the launcher's files"
[[ -f "$ENTRY" ]] || fail "no desktop entry at $ENTRY"
if command -v desktop-file-validate >/dev/null; then desktop-file-validate "$ENTRY"; fi
exe="$(sed -n 's/^Exec=\([^ ]*\).*/\1/p' "$ENTRY")"
command -v "$exe" >/dev/null || fail "the entry starts $exe, which is not on PATH"
icon="$(sed -n 's/^Icon=//p' "$ENTRY")"
compgen -G "/usr/share/icons/hicolor/*/apps/$icon.*" >/dev/null || fail "no hicolor icon named $icon"
[[ -f /usr/share/doc/overthediff/NOTICE ]] || fail "no NOTICE in /usr/share/doc/overthediff"

echo "==> start it from the desktop entry"
version="$(dpkg-query -W -f '${Version}' overthediff)"
# Set but empty: settings stay in memory (otd-settings).
export "OVERTHEDIFF_${version%%.*}_CONFIG_DIR="
if [[ -z "${XDG_RUNTIME_DIR:-}" ]]; then
  XDG_RUNTIME_DIR="$(mktemp -d)"
  export XDG_RUNTIME_DIR
fi
launch() {
  gio launch "$ENTRY"
  # The window's title is the product name; poll for it, then close the app.
  for _ in $(seq 60); do
    if xwininfo -root -tree | grep -q '"OverTheDiff"'; then
      pkill -x "$exe" || true
      return 0
    fi
    pgrep -x "$exe" >/dev/null || { echo "smoke: $exe exited without a window" >&2; return 1; }
    sleep 1
  done
  pkill -x "$exe" || true
  echo "smoke: no OverTheDiff window within 60 seconds" >&2
  return 1
}
if [[ -n "${DISPLAY:-}" ]]; then
  launch
else
  export -f launch
  export ENTRY exe
  xvfb-run -a bash -c launch
fi
echo "==> smoke test passed"
