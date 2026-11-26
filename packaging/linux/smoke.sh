#!/usr/bin/env bash
# Smoke test for the .deb: install it, check what the application launcher reads, then start the
# app through its desktop entry and wait for its window. Then remove it and check that the user's
# settings folder is kept, and that installing it again finds it.
#
#   packaging/linux/smoke.sh dist/release/overthediff-<version>-linux-<arch>.deb [OTHER.deb]
#
# With OTHER, a package of another major version, it installs OTHER first and also checks that the
# two install side by side (E01-S2-T5): both stay installed and start, `otd` runs the newer one,
# and removing the first leaves OTHER installed and starting.
#
# Run it as root or with sudo on a disposable machine (CI: Ubuntu 22.04 and 24.04), since it
# installs packages system-wide and writes a marker file into the user's settings folder. Needs
# apt, gio (libglib2.0-bin), xwininfo and xprop (x11-utils) and, with no display, xvfb-run (xvfb);
# with no GPU, Mesa's software drivers (mesa-vulkan-drivers). The app runs with its settings kept
# in memory, so it writes nothing to the user's profile itself.
set -euo pipefail
DEB="${1:?usage: packaging/linux/smoke.sh <package.deb> [<other-major.deb>]}"
OTHER="${2:-}"
# With no display, run the whole test again on a virtual X server.
if [[ -z "${DISPLAY:-}${WAYLAND_DISPLAY:-}" ]]; then exec xvfb-run -a "$0" "$@"; fi
SUDO=
[[ "$(id -u)" = 0 ]] || SUDO=sudo
if [[ -z "${XDG_RUNTIME_DIR:-}" ]]; then
  XDG_RUNTIME_DIR="$(mktemp -d)"
  export XDG_RUNTIME_DIR
fi

fail() { echo "smoke: FAIL: $*" >&2; exit 1; }
abs() { echo "$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"; }

# What a package installs, from the .deb itself: PKG, MAJOR, APP_ID, ENTRY and EXE.
describe() {
  PKG="$(dpkg-deb -f "$1" Package)"
  MAJOR="$(dpkg-deb -f "$1" Version)"
  MAJOR="${MAJOR%%.*}"
  APP_ID="io.github.alextiboldi.overthediff$MAJOR"
  ENTRY="/usr/share/applications/$APP_ID.desktop"
  EXE="otd-$MAJOR"
}

# Checks the launcher's files for the package `describe` read.
check_installed() {
  dpkg-query -W -f '${Status}' "$PKG" | grep -q 'install ok installed' || fail "$PKG is not installed"
  [[ -f "$ENTRY" ]] || fail "no desktop entry at $ENTRY"
  if command -v desktop-file-validate >/dev/null; then desktop-file-validate "$ENTRY"; fi
  [[ "$(sed -n 's/^Exec=\([^ ]*\).*/\1/p' "$ENTRY")" = "$EXE" ]] || fail "the entry doesn't start $EXE"
  command -v "$EXE" >/dev/null || fail "$EXE is not on PATH"
  icon="$(sed -n 's/^Icon=//p' "$ENTRY")"
  compgen -G "/usr/share/icons/hicolor/*/apps/$icon.*" >/dev/null || fail "no hicolor icon named $icon"
  echo "ok: $PKG is installed, with its entry, $EXE and its icon"
}

# Starts the app through its desktop entry and waits for its window, whose X11 class must be the
# app id the entry names (so the launcher matches the window to its entry), then closes it.
launch() {
  # Set but empty: settings stay in memory (otd-settings).
  export "OVERTHEDIFF_${MAJOR}_CONFIG_DIR="
  gio launch "$ENTRY"
  # The window's title is the product name: wait for it, then close the app.
  for _ in $(seq 60); do
    sleep 1
    if xwininfo -root -tree | grep '"OverTheDiff"' >/dev/null; then
      class="$(xprop -name OverTheDiff WM_CLASS)"
      pkill -x "$EXE" || true
      [[ "$class" == *"\"$APP_ID\""* ]] || fail "the window's class is $class, not $APP_ID"
      for _ in $(seq 10); do pgrep -x "$EXE" >/dev/null || break; sleep 1; done
      echo "ok: $ENTRY opened a window of class $APP_ID"
      return
    fi
    pgrep -x "$EXE" >/dev/null || fail "$EXE exited without opening a window"
  done
  pkill -x "$EXE" || true
  fail "no OverTheDiff window from $ENTRY within 60 seconds"
}

if [[ -n "$OTHER" ]]; then
  describe "$OTHER"
  OTHER_MAJOR="$MAJOR"
  echo "==> install the other major version, $OTHER"
  $SUDO apt-get install -y "$(abs "$OTHER")"
fi
describe "$DEB"
[[ -z "$OTHER" || "$MAJOR" != "$OTHER_MAJOR" ]] || fail "$DEB and $OTHER are both major version $MAJOR"

echo "==> install $DEB"
$SUDO apt-get install -y "$(abs "$DEB")"
check_installed
# Read from the package: minimal images (Docker's Ubuntu) tell dpkg to skip /usr/share/doc.
for f in NOTICE licences/UFL.txt licences/Hack-Regular.txt licences/emoji-icon-font-mit-license.txt licences/option-ext-LICENSE.txt; do
  dpkg-deb -c "$DEB" | grep "/usr/share/doc/$PKG/$f\$" >/dev/null || fail "the package has no $f"
done
newest="$MAJOR"
[[ -z "$OTHER" || "$OTHER_MAJOR" -lt "$MAJOR" ]] || newest="$OTHER_MAJOR"
[[ "$(readlink -f "$(command -v otd)")" = "/usr/bin/otd-$newest" ]] || fail "otd doesn't run the newest major version, otd-$newest"
echo "ok: otd runs otd-$newest"

echo "==> start it from the desktop entry"
launch

if [[ -n "$OTHER" ]]; then
  echo "==> the other major version is still installed and starts"
  describe "$OTHER"
  check_installed
  launch
  describe "$DEB"
fi

# The user's settings (otd-settings' folder), which removing the package must keep.
SETTINGS="${XDG_CONFIG_HOME:-$HOME/.config}/overthediff-$MAJOR"
mkdir -p "$SETTINGS"
MARKER="$SETTINGS/smoke-test-marker.txt"
echo 'kept across uninstall' >"$MARKER"

echo "==> remove $PKG"
$SUDO apt-get remove -y "$PKG"
[[ ! -e "$ENTRY" ]] || fail "removing $PKG left $ENTRY"
! command -v "$EXE" >/dev/null || fail "removing $PKG left $EXE on PATH"
[[ -f "$MARKER" ]] || fail "removing $PKG lost the settings folder $SETTINGS"
echo "ok: $PKG is removed, and $SETTINGS is kept"
if [[ -n "$OTHER" ]]; then
  [[ "$(readlink -f "$(command -v otd)")" = "/usr/bin/otd-$OTHER_MAJOR" ]] || fail "otd doesn't run otd-$OTHER_MAJOR after removing $PKG"
  echo "==> the other major version still starts"
  describe "$OTHER"
  check_installed
  launch
  describe "$DEB"
else
  ! command -v otd >/dev/null || fail "removing the only major version left otd on PATH"
fi

echo "==> install it again"
$SUDO apt-get install -y "$(abs "$DEB")"
check_installed
[[ -f "$MARKER" ]] || fail "reinstalling $PKG lost the settings folder $SETTINGS"
rm -f "$MARKER"
rmdir "$SETTINGS" 2>/dev/null || true
echo "==> smoke test passed"
