#!/usr/bin/env bash
# Build OverTheDiff.app and its disk image: $DIST/overthediff-<version>-macos-<arch>.dmg.
#
# Usage: packaging/macos/package.sh [--arch universal|aarch64|x86_64] [--skip-build]
#
# Signing (env), off until the maintainer allows it (AGENTS.md rule 4):
#   MACOS_SIGN_IDENTITY  codesign identity; import-cert.sh sets it. Unset means ad-hoc ("-"): the
#                        bundle runs on this Mac, but on another one Gatekeeper warns before it opens.
#   MACOS_KEYCHAIN       the keychain holding that identity (optional).
#   APPLE_ID, APPLE_PASSWORD, APPLE_TEAM_ID  all three, with a real identity, notarise and staple.
set -euo pipefail
# shellcheck source=../env.sh
. "$(dirname "${BASH_SOURCE[0]}")/../env.sh"
HERE="$ROOT/packaging/macos"

ARCH=universal
SKIP_BUILD=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --arch) ARCH="${2:?--arch needs a value}"; shift 2 ;;
    --skip-build) SKIP_BUILD=1; shift ;;
    -h | --help) sed -n '2,11p' "$0"; exit 0 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done
case "$ARCH" in
  universal) TARGETS=(aarch64-apple-darwin x86_64-apple-darwin) ;;
  aarch64 | x86_64) TARGETS=("$ARCH-apple-darwin") ;;
  *) echo "unknown --arch $ARCH" >&2; exit 2 ;;
esac

# The same as LSMinimumSystemVersion in Info.plist.in (E01-S2 Decisions).
export MACOSX_DEPLOYMENT_TARGET=11.0
IDENTITY="${MACOS_SIGN_IDENTITY:--}"
WORK="$CARGO_TARGET_DIR/macos-package"
APP="$WORK/OverTheDiff.app"
DMG="$DIST/overthediff-$VERSION-macos-$ARCH.dmg"

NOTARIZE=0
if [[ "$IDENTITY" = "-" ]]; then
  warn "macOS: ad-hoc signed, not notarised: Gatekeeper warns when this app opens on another Mac"
elif [[ -n "${APPLE_ID:-}" && -n "${APPLE_PASSWORD:-}" && -n "${APPLE_TEAM_ID:-}" ]]; then
  NOTARIZE=1
else
  warn "macOS: APPLE_ID, APPLE_PASSWORD or APPLE_TEAM_ID is missing: signed but not notarised"
fi
echo "==> OverTheDiff $VERSION for macOS ($ARCH), identity $IDENTITY, notarise $NOTARIZE"

if [[ "$SKIP_BUILD" = 0 ]]; then
  args=()
  for t in "${TARGETS[@]}"; do args+=(--target "$t"); done
  (cd "$ROOT" && cargo build --release --locked -p otd "${args[@]}")
fi

rm -rf "$WORK"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
slices=()
for t in "${TARGETS[@]}"; do slices+=("$CARGO_TARGET_DIR/$t/release/otd"); done
lipo -create -output "$APP/Contents/MacOS/otd" "${slices[@]}"
lipo -info "$APP/Contents/MacOS/otd"
copy_notices "$APP/Contents/Resources"
sed -e "s/@VERSION@/$VERSION/g" -e "s/@SHORT_VERSION@/${VERSION%%-*}/g" "$HERE/Info.plist.in" >"$APP/Contents/Info.plist"
plutil -lint "$APP/Contents/Info.plist"
printf 'APPL????' >"$APP/Contents/PkgInfo"

sign() {
  # A real identity gets the hardened runtime and a secure timestamp, which notarisation needs;
  # an ad-hoc signature can have neither.
  local opts=(--timestamp=none)
  [[ "$IDENTITY" = "-" ]] || opts=(--timestamp --options runtime)
  [[ -z "${MACOS_KEYCHAIN:-}" ]] || opts+=(--keychain "$MACOS_KEYCHAIN")
  codesign --force --sign "$IDENTITY" "${opts[@]}" "$@"
}

notarize() {
  local out
  echo "==> notarising $(basename "$1")"
  out="$(xcrun notarytool submit "$1" --apple-id "$APPLE_ID" --password "$APPLE_PASSWORD" \
    --team-id "$APPLE_TEAM_ID" --wait --timeout 1h --output-format json)" || true
  echo "$out"
  if [[ "$(printf '%s' "$out" | plutil -extract status raw -o - - 2>/dev/null)" != "Accepted" ]]; then
    echo "error: notarisation of $(basename "$1") was not accepted" >&2
    exit 1
  fi
}

# The executable is the bundle's only code, so signing the bundle signs it too.
sign "$APP"
codesign --verify --strict --verbose=2 "$APP"
if [[ "$NOTARIZE" = 1 ]]; then
  ditto -c -k --keepParent "$APP" "$WORK/notarize.zip"
  notarize "$WORK/notarize.zip"
  xcrun stapler staple "$APP"
fi

echo "==> $DMG"
STAGE="$WORK/dmg"
mkdir -p "$STAGE"
ditto "$APP" "$STAGE/OverTheDiff.app"
ln -s /Applications "$STAGE/Applications"
rm -f "$DMG"
# Not `makehybrid -hfs`: it gives text files a Finder type, an extended attribute that breaks the
# bundle's signature once the app is copied out of the image. `create` can fail with "Resource
# busy" on a busy runner, so it gets three tries.
for attempt in 1 2 3; do
  if hdiutil create -volname OverTheDiff -srcfolder "$STAGE" -fs HFS+ -format UDZO -imagekey zlib-level=9 -ov "$DMG"; then
    break
  fi
  [[ "$attempt" -lt 3 ]] || exit 1
  warn "hdiutil create failed (try $attempt of 3); trying again in 10 seconds"
  sleep 10
done
sign "$DMG"
if [[ "$NOTARIZE" = 1 ]]; then notarize "$DMG" && xcrun stapler staple "$DMG"; fi
ls -lh "$DMG"
