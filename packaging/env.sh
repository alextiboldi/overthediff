# shellcheck shell=bash
# Shared setup for the packaging scripts; source it: `. "$(dirname "$0")/../env.sh"`.
#
# Exports ROOT (the workspace), VERSION ([workspace.package] version in Cargo.toml, or OTD_VERSION),
# DIST (where release files go, default $ROOT/dist/release) and CARGO_TARGET_DIR. The release
# workflow reads the OTD_ prefix from the VERSION line below, so keep its shape.

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export ROOT

VERSION="${OTD_VERSION:-$(awk '/^\[/ { pkg = ($0 == "[workspace.package]"); next } pkg && $1 == "version" { gsub(/"/, "", $3); print $3; exit }' "$ROOT/Cargo.toml")}"
if [[ -z "$VERSION" ]]; then
  echo "error: no [workspace.package] version in $ROOT/Cargo.toml" >&2
  exit 1
fi
export VERSION

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
