#!/usr/bin/env bash
# Check a disk image package.sh built, on the Mac this runs on (the E01-S2 smoke test):
#   - the image and the app inside it have valid signatures;
#   - the executable has a slice for each architecture --arch names;
#   - NOTICE ships in the bundle;
#   - each slice this Mac can run starts and answers: natively for this Mac's own architecture,
#     and the Intel slice under Rosetta on Apple silicon when Rosetta is installed;
#   - unless the signature is ad-hoc, Gatekeeper accepts the app and the image and both carry a
#     notarisation ticket. An ad-hoc build skips this and says so: Gatekeeper rejects it.
#
# Usage: packaging/macos/verify.sh [--arch universal|aarch64|x86_64] [path/to/image.dmg]
set -euo pipefail
# shellcheck source=../env.sh
. "$(dirname "${BASH_SOURCE[0]}")/../env.sh"

ARCH=universal
DMG=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --arch) ARCH="${2:?--arch needs a value}"; shift 2 ;;
    -h | --help) sed -n '2,12p' "$0"; exit 0 ;;
    *) DMG="$1"; shift ;;
  esac
done
case "$ARCH" in
  universal) WANT=(arm64 x86_64) ;;
  aarch64) WANT=(arm64) ;;
  x86_64) WANT=(x86_64) ;;
  *) echo "unknown --arch $ARCH" >&2; exit 2 ;;
esac
DMG="${DMG:-$DIST/overthediff-$VERSION-macos-$ARCH.dmg}"

fail() { echo "FAIL: $*" >&2; exit 1; }

codesign --verify --strict --verbose=2 "$DMG"
MOUNT="$(mktemp -d "${TMPDIR:-/tmp}/otd-verify.XXXXXXXX")"
trap 'hdiutil detach "$MOUNT" -quiet || true; rmdir "$MOUNT" 2>/dev/null || true' EXIT
hdiutil attach "$DMG" -readonly -nobrowse -noautoopen -mountpoint "$MOUNT" -quiet
APP="$MOUNT/OverTheDiff.app"
EXE="$APP/Contents/MacOS/$(plutil -extract CFBundleExecutable raw -o - "$APP/Contents/Info.plist")"
codesign --verify --strict --deep --verbose=2 "$APP"
[[ -f "$APP/Contents/Resources/NOTICE" ]] || fail "NOTICE is not in the bundle"

have=" $(lipo -archs "$EXE") "
for a in "${WANT[@]}"; do
  [[ "$have" == *" $a "* ]] || fail "the executable has no $a slice (it has:$have)"
done
echo "ok: slices$have"

# An argument no feature claims makes otd print its usage and exit before it opens a window,
# reads settings or looks for a running instance. Any exit of its own counts (not a crash, a
# signal, or `arch` failing to start the slice): the usage text and codes belong to E01-S1.
run_slice() {
  local slice="$1" how="$2" out code=0
  # Empty keeps settings in memory, so a change to the launch order can't write the real profile.
  out="$(env "OVERTHEDIFF_${VERSION%%.*}_CONFIG_DIR=" arch "-$slice" "$EXE" -packaging-smoke-test 2>&1)" || code=$?
  if [[ "$code" -ge 1 && "$code" -lt 128 && "$out" == otd:* ]]; then
    echo "ok: the $slice slice runs $how"
  else
    fail "the $slice slice didn't run $how (exit $code): $out"
  fi
}
if [[ "$(sysctl -n hw.optional.arm64 2>/dev/null || echo 0)" = 1 ]]; then native=arm64; else native=x86_64; fi
ran_native=0
for a in "${WANT[@]}"; do
  if [[ "$a" = "$native" ]]; then
    run_slice "$a" natively
    ran_native=1
  elif [[ "$a" = x86_64 ]] && arch -x86_64 /usr/bin/true 2>/dev/null; then
    run_slice "$a" "under Rosetta"
  else
    warn "this $native Mac can't run the $a slice; run this check on a $a Mac too"
  fi
done
[[ "$ran_native" = 1 ]] || warn "this Mac is $native, so no slice was checked natively"

# Only an ad-hoc signature skips Gatekeeper's check; any other signature must pass it.
signature="$(codesign -dv "$APP" 2>&1)"
if [[ "$signature" == *"Signature=adhoc"* ]]; then
  warn "ad-hoc signature: Gatekeeper's check is skipped, and on another Mac the app opens only after a warning"
else
  spctl --assess --type execute -vv "$APP"
  spctl --assess --type open --context context:primary-signature -vv "$DMG"
  xcrun stapler validate "$APP"
  xcrun stapler validate "$DMG"
  echo "ok: Gatekeeper accepts the app and the image, and both are notarised"
fi
