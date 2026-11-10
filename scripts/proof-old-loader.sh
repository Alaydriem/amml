#!/usr/bin/env bash
# Proves that the released 26.60.5 loader refuses a manifest with `amml_version`.
# That refusal is the fail-safe the registry floor (26.60.6) relies on: a mod with the key never runs on 26.60.5.
# Everything stays under target/proof-old-loader; no file outside it changes.
# Needs: the local git tag 26.60.5 (a shallow or tagless clone fails), the crates of the 26.60.5 lockfile in the
# cargo cache (the test runs with --offline), and no second run at the same time (both share target/proof-old-loader).
set -u

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
base="$root/target/proof-old-loader"
src="$base/src"
out="$base/output.txt"
tag="26.60.5"
mkdir -p "$base"

if [ ! -e "$src/.git" ]; then
  # after a cargo clean the folder is gone but git still lists the worktree, and worktree add would refuse
  git -C "$root" worktree prune
  git -C "$root" worktree add --detach "$src" "$tag" || { echo "cannot create the worktree of $tag at $src"; exit 1; }
fi
head_tag="$(git -C "$src" describe --tags --exact-match HEAD 2>/dev/null || true)"
if [ "$head_tag" != "$tag" ]; then
  echo "$src is not at tag $tag (it is at '${head_tag:-no tag}'); remove it with: git worktree remove --force $src"
  exit 1
fi

# The test and its target exist only in the worktree.
git -C "$src" checkout -- loader/runtime/Cargo.toml || exit 1
mkdir -p "$src/loader/runtime/tests"
cat > "$src/loader/runtime/tests/proof_old_loader.rs" <<'RUST'
use std::fs;

use amml_runtime::ModRegistry;

#[test]
fn the_released_loader_refuses_amml_version() {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("@proof").join("gated");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("gated.dll"), b"").unwrap();
    let manifest = "namespace = \"proof\"\nname = \"gated\"\nentry = \"gated.dll\"\napi_version = 2\namml_version = \"26.60.6\"\n";
    fs::write(dir.join("amml.hcl"), manifest).unwrap();

    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(root.path()).unwrap();

    assert!(scan.mods.is_empty(), "26.60.5 / gated 2.0.0 / scan: the mod must not be accepted");
    assert_eq!(scan.skipped.len(), 1, "26.60.5 / gated 2.0.0 / scan");
    assert_eq!(scan.skipped[0].reason, "unknown attribute `amml_version`", "26.60.5 / gated 2.0.0 / scan");
    println!("PROOF-OLD-LOADER-REFUSES");
}
RUST
cat >> "$src/loader/runtime/Cargo.toml" <<'TOML'

[[test]]
name = "proof_old_loader"
path = "tests/proof_old_loader.rs"
TOML

status=0
(cd "$src" && CARGO_TARGET_DIR="$base/target" cargo test --offline -p amml-runtime --test proof_old_loader -- --nocapture) > "$out" 2>&1 || status=$?
cat "$out"
if [ "$status" -ne 0 ]; then
  echo "FAIL: the old-loader test did not build or did not pass (exit $status)"
  exit 1
fi
if ! grep -q -x "PROOF-OLD-LOADER-REFUSES" "$out" || ! grep -q "test result: ok. 1 passed" "$out"; then
  echo "FAIL: the old-loader test output lacks its proof line or a passing result"
  exit 1
fi
