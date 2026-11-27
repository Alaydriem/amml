#!/usr/bin/env bash
# Packages the release build in target/release as a local AMML zip for `amml-launcher instance new --amml-zip`.
#
# Usage: scripts/package-local-zip.sh <out.zip>
#
# Run `mise run build-lib` first. The zip has the layout of a release zip (amml.dll, amml-patch.exe, amml/recipes/,
# amml/data/, amml/amml-version.txt) plus amml/amml-api.txt, which a local zip must carry.
set -eu

out="${1:-}"
[ -n "$out" ] || { echo "error: missing the output zip path" >&2; exit 1; }
case "$out" in /*|?:*) ;; *) out="$PWD/$out" ;; esac

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

for built in target/release/amml.dll target/release/amml-patch.exe; do
  [ -f "$built" ] || { echo "error: $built is missing; run mise run build-lib" >&2; exit 1; }
done

version="$(sed -nE '/^\[workspace\.package\]/,/^\[/{s/^version *= *"([^"]+)".*/\1/p}' Cargo.toml)"
api="$(sed -nE 's/^pub const AMML_API_VERSION: u32 = ([0-9]+);.*/\1/p' loader/abi/src/amml_api.rs)"
[ -n "$version" ] && [ -n "$api" ] || { echo "error: the workspace version or AMML_API_VERSION was not found" >&2; exit 1; }

stage="$(mktemp -d)"
trap 'rm -rf "$stage"' EXIT
mkdir -p "$stage/amml/data"
cp target/release/amml.dll target/release/amml-patch.exe "$stage/"
cp -r recipes "$stage/amml/recipes"
cp data/*.hcl "$stage/amml/data/"
printf '%s' "$version" > "$stage/amml/amml-version.txt"
printf '%s' "$api" > "$stage/amml/amml-api.txt"

rm -f "$out"
(cd "$stage" && "${SYSTEMROOT:-/c/Windows}/System32/tar.exe" -a -c -f "$out" amml.dll amml-patch.exe amml)
echo "wrote $out (AMML $version, API $api)"
