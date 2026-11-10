use std::fs;

use amml_proof_fixture::{DummyMod, ProofTree};
use serde_json::Value;

type Rows = Vec<(String, Option<String>)>;

fn read(proof: &ProofTree, path: &str) -> Value {
    serde_json::from_slice(&fs::read(proof.tree.join(path)).unwrap()).unwrap()
}

fn rows(list: &Value) -> Rows {
    list.as_array()
        .unwrap()
        .iter()
        .map(|r| {
            (r["version"].as_str().unwrap().to_string(), r.get("min_amml").map(|v| v.as_str().unwrap().to_string()))
        })
        .collect()
}

fn find(list: &Rows, version: &str) -> Option<String> {
    list.iter().find(|(v, _)| v == version).unwrap_or_else(|| panic!("{version} missing")).1.clone()
}

#[test]
fn tree_lists_min_amml_per_release() {
    let work = tempfile::tempdir().unwrap();
    let proof = ProofTree::build(work.path());

    let gated = rows(&read(&proof, "mods/@proof/gated.json")["versions"]);
    assert_eq!(gated.len(), 2, "gated releases");
    assert_eq!(find(&gated, "1.0.0"), None, "gated 1.0.0 has no min_amml");
    assert_eq!(find(&gated, "2.0.0"), Some("26.60.6".to_string()), "gated 2.0.0");
    let raw = fs::read_to_string(proof.tree.join("mods/@proof/gated.json")).unwrap();
    assert_eq!(raw.matches("min_amml").count(), 1, "the key is absent, not null, on gated 1.0.0");

    let onlygated = rows(&read(&proof, "mods/@proof/onlygated.json")["versions"]);
    assert_eq!(find(&onlygated, "1.0.0"), Some("26.62.0".to_string()), "onlygated 1.0.0");

    let plain = rows(&read(&proof, "mods/@proof/plain.json")["versions"]);
    assert_eq!(plain.len(), 2, "plain releases");
    assert!(plain.iter().all(|(_, min)| min.is_none()), "plain has no min_amml");
}

#[test]
fn index_cards_carry_min_amml_and_releases() {
    let work = tempfile::tempdir().unwrap();
    let proof = ProofTree::build(work.path());
    let page = read(&proof, "index/api-2/0001.json");
    let cards = page["cards"].as_array().unwrap();
    let card = |id: &str| cards.iter().find(|c| c["id"] == id).unwrap_or_else(|| panic!("{id} has no card"));

    let gated = card("@proof/gated");
    assert_eq!(gated["version"], "2.0.0");
    assert_eq!(gated["min_amml"], "26.60.6");
    let releases = rows(&gated["releases"]);
    assert_eq!(releases.len(), 2, "gated card releases");
    assert_eq!(find(&releases, "1.0.0"), None);
    assert_eq!(find(&releases, "2.0.0"), Some("26.60.6".to_string()));

    let plain = card("@proof/plain");
    assert!(plain.get("releases").is_none(), "plain card has no releases key");
    assert!(plain.get("min_amml").is_none(), "plain card has no min_amml");

    let onlygated = card("@proof/onlygated");
    assert_eq!(onlygated["version"], "1.0.0");
    assert_eq!(onlygated["min_amml"], "26.62.0", "onlygated card min_amml");

    // names.json is {"schema": 2, "rows": [[id, title, ...], ...]}
    let names = read(&proof, "names.json");
    let mut listed: Vec<String> = names["rows"]
        .as_array()
        .unwrap_or_else(|| panic!("names.json has no rows: {names}"))
        .iter()
        .map(|row| row[0].as_str().unwrap_or_else(|| panic!("names.json row {row}")).to_string())
        .collect();
    listed.sort();
    assert_eq!(listed, ["@proof/gated", "@proof/onlygated", "@proof/plain"], "names.json ids");
}

#[test]
fn every_asset_url_is_local() {
    let work = tempfile::tempdir().unwrap();
    let proof = ProofTree::build(work.path());
    let base = proof.server.base();

    let root = read(&proof, "registry.json");
    let amml = root["amml"]["versions"].as_array().unwrap();
    assert_eq!(amml.len(), 2, "both AMML releases are listed");
    for version in amml {
        let asset = version["asset"].as_str().unwrap();
        assert!(asset.starts_with(&base), "amml asset {asset}");
    }
    for name in ["plain", "gated", "onlygated"] {
        let doc = read(&proof, &format!("mods/@proof/{name}.json"));
        for release in doc["versions"].as_array().unwrap() {
            let asset = release["asset"].as_str().unwrap();
            assert!(asset.starts_with(&base), "{name} asset {asset}");
        }
    }
    assert_eq!(proof.registry_base, format!("{base}/registry"));
    for dummy in DummyMod::MATRIX {
        assert!(proof.mod_zip(dummy.name, dummy.version).is_file(), "{} {}", dummy.name, dummy.version);
    }
    let requests = proof.server.requests();
    assert!(!requests.is_empty());
    assert!(requests.iter().all(|path| path.starts_with("/gh/")), "{requests:?}");
}

#[test]
fn refusals_write_nothing() {
    let work = tempfile::tempdir().unwrap();
    let proof = ProofTree::build(work.path());
    let cases = [
        ("26.63.0", "the newest listed AMML is 26.62.0"),
        ("26.60.5", "26.60.6 or newer"),
        ("26.062.0", "MAJOR.MINOR.PATCH"),
    ];
    for (minimum, reason) in cases {
        let before = proof.snapshot();
        let dummy = DummyMod { name: "plain", version: "3.0.0", amml_version: Some(minimum) };
        let output = proof.try_add_mod(&dummy);
        assert_eq!(output.code, 1, "{minimum}: {}", output.stderr);
        assert!(output.stderr.contains(minimum), "{minimum}: {}", output.stderr);
        assert!(output.stderr.contains(reason), "{minimum}: {}", output.stderr);
        assert_eq!(proof.snapshot(), before, "{minimum} changed the tree");
    }

    let old_work = tempfile::tempdir().unwrap();
    let old = ProofTree::build_old_only(old_work.path());
    let before = old.snapshot();
    let dummy = DummyMod { name: "plain", version: "3.0.0", amml_version: Some("26.60.6") };
    let output = old.try_add_mod(&dummy);
    assert_eq!(output.code, 1, "{}", output.stderr);
    assert!(output.stderr.contains("the newest listed AMML is 26.60.5"), "{}", output.stderr);
    assert_eq!(old.snapshot(), before, "old-only tree changed");
}
