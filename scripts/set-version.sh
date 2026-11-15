#!/usr/bin/env bash
# Writes the release version into the workspace and checks that every crate has it.
#
# Usage: scripts/set-version.sh <version>
#   version  release version, Cargo semver (26.60.5)
#
# Rewrites the version in [workspace.package] of Cargo.toml. Cargo.lock records each workspace crate's version,
# so cargo metadata rewrites it. Every workspace crate must then report the version; only the test fixture
# crates keep their own.
set -eu

version="${1:-}"

fail() {
  echo "error: $1" >&2
  exit 1
}

[ -n "$version" ] || fail "missing the version"

semver='^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(-[0-9A-Za-z.-]+)?$'
[[ "$version" =~ $semver ]] || fail "version '$version' is not Cargo semver (for example 26.60.1)"

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

sed -i "/^\[workspace\.package\]/,/^\[/ s/^version = \".*\"/version = \"$version\"/" Cargo.toml
if ! sed -n '/^\[workspace\.package\]/,/^\[/p' Cargo.toml | grep -qF "version = \"$version\""; then
  fail "Cargo.toml has no version line in [workspace.package]"
fi

cargo metadata --format-version 1 > /dev/null || fail "cargo metadata failed"
wrong="$(cargo metadata --no-deps --format-version 1 \
  | jq -r --arg v "$version" '.packages[]
      | select(.manifest_path | gsub("\\\\"; "/") | contains("/tests/fixtures/") | not)
      | select(.version != $v)
      | "\(.name) \(.version)"')"
if [ -n "$wrong" ]; then
  echo "$wrong" >&2
  fail "these crates do not have version $version (listed above)"
fi

grep -qF "version = \"$version\"" Cargo.lock || fail "Cargo.lock does not record version $version"
echo "version $version set"
