# shellcheck shell=bash
# Shared setup for the packaging scripts; source it: `. "$(dirname "$0")/../env.sh"`.
#
# Exports ROOT (the workspace), VERSION ([workspace.package] version in Cargo.toml, or OTD_VERSION),
# MAJOR (VERSION's major), APP_ID (the reverse-DNS id of that major version), DIST (where release
# files go, default $ROOT/dist/release) and CARGO_TARGET_DIR. The release workflow reads the OTD_
# prefix from the VERSION line below, so keep its shape.
#
# Each major version installs side by side with the others (E01-S2-T5): every name and path a
# package installs carries MAJOR, as the settings folder (overthediff-<major>) already does.

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export ROOT

VERSION="${OTD_VERSION:-$(awk '/^\[/ { pkg = ($0 == "[workspace.package]"); next } pkg && $1 == "version" { gsub(/"/, "", $3); print $3; exit }' "$ROOT/Cargo.toml")}"
if [[ -z "$VERSION" ]]; then
  echo "error: no [workspace.package] version in $ROOT/Cargo.toml" >&2
  exit 1
fi
export VERSION

# The binary's settings folder and app id take the major from Cargo.toml when it is compiled, so a
# package labelled with another major would install under names its binary doesn't use.
MAJOR="${VERSION%%.*}"
CARGO_VERSION="$(awk '/^\[/ { pkg = ($0 == "[workspace.package]"); next } pkg && $1 == "version" { gsub(/"/, "", $3); print $3; exit }' "$ROOT/Cargo.toml")"
if [[ "$MAJOR" != "${CARGO_VERSION%%.*}" ]]; then
  echo "error: OTD_VERSION $VERSION has another major version than Cargo.toml's $CARGO_VERSION, which the binary is built with" >&2
  exit 1
fi
# The same as APP_ID in apps/otd/src/main.rs.
APP_ID="io.github.alextiboldi.overthediff$MAJOR"
export MAJOR APP_ID

export DIST="${DIST:-$ROOT/dist/release}"
mkdir -p "$DIST"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"

# A GitHub Actions warning, or a plain one on stderr outside Actions.
warn() {
  if [[ -n "${GITHUB_ACTIONS:-}" ]]; then echo "::warning::$*"; else echo "warning: $*" >&2; fi
}

# The notices and licence texts every package carries.
copy_notices() {
  local dest="$1" f
  for f in NOTICE ATTRIBUTION.md LICENSE LICENSE-MIT LICENSE-APACHE; do
    if [[ -f "$ROOT/$f" ]]; then cp "$ROOT/$f" "$dest/"; fi
  done
}
