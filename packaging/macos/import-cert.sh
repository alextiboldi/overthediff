#!/usr/bin/env bash
# CI only: put the Developer ID certificate in a temporary keychain for package.sh to sign with.
#
# Inputs (env): APPLE_CERTIFICATE (a base64 .p12), APPLE_CERTIFICATE_PASSWORD, KEYCHAIN_PASSWORD
# (random when unset), RUNNER_TEMP. Outputs MACOS_SIGN_IDENTITY and MACOS_KEYCHAIN to $GITHUB_ENV.
# Without APPLE_CERTIFICATE it warns and exits 0, and package.sh signs ad-hoc.
set -euo pipefail
umask 077
# shellcheck source=../env.sh
. "$(dirname "${BASH_SOURCE[0]}")/../env.sh"

if [[ -z "${APPLE_CERTIFICATE:-}" ]]; then
  warn "APPLE_CERTIFICATE is not set: the macOS build is ad-hoc signed and not notarised"
  exit 0
fi

# A private folder made by mktemp, so nobody else's file or link at a guessable name receives the
# decoded certificate.
WORK="$(mktemp -d "${RUNNER_TEMP:-${TMPDIR:-/tmp}}/otd-signing.XXXXXXXX")"
KEYCHAIN="$WORK/otd-signing.keychain-db"
CERT="$WORK/otd-signing.p12"
KC_PASS="${KEYCHAIN_PASSWORD:-$(openssl rand -hex 24)}"
trap 'rm -f "$CERT"' EXIT

printf '%s' "$APPLE_CERTIFICATE" | base64 --decode >"$CERT"
security create-keychain -p "$KC_PASS" "$KEYCHAIN"
security set-keychain-settings -lut 21600 "$KEYCHAIN"
security unlock-keychain -p "$KC_PASS" "$KEYCHAIN"
security import "$CERT" -P "${APPLE_CERTIFICATE_PASSWORD:-}" -T /usr/bin/codesign -f pkcs12 -k "$KEYCHAIN"
security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k "$KC_PASS" "$KEYCHAIN" >/dev/null
# shellcheck disable=SC2046 # the existing keychains, one word each
security list-keychains -d user -s "$KEYCHAIN" $(security list-keychains -d user | tr -d '"')

IDENTITY="$(security find-identity -v -p codesigning "$KEYCHAIN" | awk '/Developer ID Application/ { print $2; exit }')"
if [[ -z "$IDENTITY" ]]; then
  echo "error: APPLE_CERTIFICATE holds no Developer ID Application identity" >&2
  exit 1
fi
{ echo "MACOS_SIGN_IDENTITY=$IDENTITY"; echo "MACOS_KEYCHAIN=$KEYCHAIN"; } >>"${GITHUB_ENV:-/dev/stdout}"
