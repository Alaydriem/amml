#!/usr/bin/env bash
# Rebuilds the tracked files of the sibling ../amml-registry checkout with build-tree and compares the result byte for byte.
# Everything stays under target/registry-proof; the registry checkout is only read.
set -u
set -o pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
registry="${AMML_REGISTRY_DIR:-$root/../amml-registry}"
base="$root/target/registry-proof"

if [ ! -e "$registry/.git" ]; then
  echo "registry-proof: no git checkout at $registry (set AMML_REGISTRY_DIR)"
  exit 1
fi

# Local refs only (no fetch): a checkout behind its upstream would prove an old registry.
if git -C "$registry" rev-parse --verify -q '@{upstream}' > /dev/null; then
  behind="$(git -C "$registry" rev-list --count 'HEAD..@{upstream}')"
  if [ "$behind" -gt 0 ]; then
    echo "registry-proof: $registry is $behind commit(s) behind its upstream; pull it first (git -C $registry pull), then run this again"
    exit 1
  fi
fi
echo "registry-proof: archiving $(git -C "$registry" rev-parse HEAD)"

rm -rf "$base"
mkdir -p "$base/before" "$base/after"
git -C "$registry" archive HEAD | tar -x -C "$base/before" || { echo "registry-proof: git archive failed"; exit 1; }
cp -R "$base/before/." "$base/after/" || exit 1

cargo run -q -p amml-registry -- build-tree --tree "$base/after" --allow "$base/after/mods.hcl" || {
  echo "registry-proof: build-tree failed"
  exit 1
}

if git -c core.autocrlf=false diff --no-index --exit-code --stat "$base/before" "$base/after"; then
  echo "registry-proof: identical"
else
  echo "registry-proof: the rebuild differs from the checkout"
  exit 1
fi
