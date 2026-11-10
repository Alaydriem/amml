#!/usr/bin/env bash
# Builds the release loader and patcher and packages the AMML release zip.
#
# Usage: scripts/package-release.sh <version> <game-versions>
#   version        release version, Cargo semver (26.60.5)
#   game-versions  tested Minecraft versions, comma separated (1.26.60.5)
#
# Writes, in the repository root:
#   amml-<version>-windows-x64.zip   amml.dll, amml-patch.exe, amml/recipes/, amml/data/ (data/*.hcl), amml/amml-version.txt
#   SHA256SUMS.txt                   the checksum of the zip
#   release-meta.env                 API_VERSION=<n> and GAME_VERSIONS=<list>
#
# Nothing is built or written when an argument or the API version constant is invalid.
set -eu

version="${1:-}"
game_versions="${2:-}"

fail() {
  echo "error: $1" >&2
  exit 1
}

[ -n "$version" ] || fail "missing the release version"
[ -n "$game_versions" ] || fail "missing the tested game versions"

semver='^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(-[0-9A-Za-z.-]+)?$'
[[ "$version" =~ $semver ]] || fail "version '$version' is not Cargo semver (for example 26.60.5)"

game_re='^[0-9]+(\.[0-9]+){3}(,[0-9]+(\.[0-9]+){3})*$'
if ! [[ "$game_versions" =~ $game_re ]]; then
  IFS=',' read -r -a listed <<< "$game_versions,"
  for entry in "${listed[@]}"; do
    [[ "$entry" =~ ^[0-9]+(\.[0-9]+){3}$ ]] || fail "game version '$entry' does not match <major>.<minor>.<patch>.<build> (for example 1.26.60.5)"
  done
  fail "game versions '$game_versions' are not a comma separated list of <major>.<minor>.<patch>.<build>"
fi

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

api_line="$(grep -E '^pub const AMML_API_VERSION: u32 = [0-9]+;' loader/abi/src/amml_api.rs || true)"
[ -n "$api_line" ] || fail "AMML_API_VERSION was not found in loader/abi/src/amml_api.rs"
api_version="$(echo "$api_line" | sed -E 's/.*= ([0-9]+);.*/\1/')"

zip="amml-$version-windows-x64.zip"
rm -rf stage
rm -f "$zip" SHA256SUMS.txt release-meta.env

cargo build --release -p amml-loader -p amml-patch

mkdir -p stage/amml
cp target/release/amml.dll stage/amml.dll
cp target/release/amml-patch.exe stage/amml-patch.exe
cp -r recipes stage/amml/recipes
mkdir -p stage/amml/data
cp data/*.hcl stage/amml/data/
printf '%s' "$version" > stage/amml/amml-version.txt

if command -v 7z > /dev/null 2>&1; then
  (cd stage && 7z a -tzip "../$zip" -- amml.dll amml-patch.exe amml)
elif [ -x "${SYSTEMROOT:-/c/Windows}/System32/tar.exe" ]; then
  # Windows ships bsdtar, which writes a zip when the file name ends in .zip.
  (cd stage && "${SYSTEMROOT:-/c/Windows}/System32/tar.exe" -a -c -f "../$zip" amml.dll amml-patch.exe amml)
else
  fail "neither 7z nor the Windows tar.exe was found to write the zip"
fi
rm -rf stage

sha256sum "$zip" > SHA256SUMS.txt
printf 'API_VERSION=%s\nGAME_VERSIONS=%s\n' "$api_version" "$game_versions" > release-meta.env
cat SHA256SUMS.txt
cat release-meta.env
