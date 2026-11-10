use crate::support::ProofHarness;
use amml_launcher_core::{
    InstanceReader, LauncherBlockStore, LauncherError, ModChange, NeedsAmml, NewInstanceOutcome, UpdateRow,
    UpdateStatus,
};
use amml_proof_fixture::{DummyMod, ProofTree};
use serde_json::Value;

const OLD_GAME: &str = ProofTree::OLD_GAME;
const NEW_GAME: &str = ProofTree::NEW_GAME;

fn cell(game: &str, text: &str) -> String {
    let amml = if game == OLD_GAME { ProofTree::OLD_AMML } else { ProofTree::NEW_AMML };
    format!("{amml} / {text}")
}

fn json(harness: &ProofHarness, path: &str) -> Value {
    serde_json::from_slice(&std::fs::read(harness.proof.tree.join(path)).unwrap()).unwrap()
}

fn min_amml(harness: &ProofHarness, name: &str, version: &str) -> Option<String> {
    let doc = json(harness, &format!("mods/@proof/{name}.json"));
    let release = doc["versions"].as_array().unwrap().iter().find(|r| r["version"] == version).unwrap().clone();
    release.get("min_amml").map(|value| value.as_str().unwrap().to_string())
}

// counts the asset urls of a json value and asserts each one is on the local server
fn count_assets(value: &Value, base: &str) -> usize {
    match value {
        Value::Object(map) => map
            .iter()
            .map(|(key, inner)| match (key.as_str(), inner) {
                ("asset", Value::String(url)) => {
                    assert!(url.starts_with(&format!("{base}/")), "tree / asset / {url} is not on {base}");
                    1
                }
                _ => count_assets(inner, base),
            })
            .sum(),
        Value::Array(items) => items.iter().map(|inner| count_assets(inner, base)).sum(),
        _ => 0,
    }
}

// the tree must say what the matrix says, before any launcher outcome is judged
fn assert_tree(harness: &ProofHarness) {
    for dummy in DummyMod::MATRIX {
        assert_eq!(
            min_amml(harness, dummy.name, dummy.version),
            dummy.amml_version.map(str::to_string),
            "tree / {} {} / min_amml",
            dummy.name,
            dummy.version
        );
    }
    let base = harness.proof.server.base();
    let mut assets = 0;
    for (path, bytes) in harness.proof.snapshot() {
        if path.extension().is_some_and(|ext| ext == "json") {
            let value: Value = serde_json::from_slice(&bytes).unwrap();
            assets += count_assets(&value, &base);
        }
    }
    assert!(assets >= 7, "tree / asset urls / expected at least 7 assets, found {assets}");
}

fn needs(version: &str, game: Option<&str>) -> NeedsAmml {
    NeedsAmml { version: version.to_string(), game: game.map(str::to_string) }
}

fn row<'a>(rows: &'a [UpdateRow], id: &str) -> &'a UpdateRow {
    rows.iter().find(|row| row.id == id).unwrap_or_else(|| panic!("no row for {id}"))
}

fn recorded_amml(harness: &ProofHarness, name: &str) -> String {
    InstanceReader::read(&harness.paths.instance(name)).unwrap().amml.version.unwrap()
}

fn amml_of(game: &str) -> &'static str {
    if game == OLD_GAME { ProofTree::OLD_AMML } else { ProofTree::NEW_AMML }
}

fn installed(outcome: &NewInstanceOutcome, id: &str) -> Option<String> {
    outcome.state.mods.iter().find(|installed| installed.id.as_str() == id).and_then(|m| m.version.clone())
}

#[test]
fn instances_get_the_amml_of_their_game() {
    let harness = ProofHarness::new();
    assert_tree(&harness);
    for game in [OLD_GAME, NEW_GAME] {
        let name = harness.instance_on(game);
        let state = InstanceReader::read(&harness.paths.instance(&name)).unwrap();
        assert_eq!(state.amml.version.as_deref(), Some(amml_of(game)), "{}", cell(game, "amml / create"));
        let block = LauncherBlockStore::read(&state.folder).unwrap();
        assert_eq!(block.amml.as_deref(), Some(amml_of(game)), "{}", cell(game, "amml record / create"));
    }
}

#[test]
fn new_instance_defaults() {
    let harness = ProofHarness::new();
    assert_tree(&harness);

    let old = harness.create(OLD_GAME, "old").unwrap();
    assert_eq!(recorded_amml(&harness, "old"), ProofTree::OLD_AMML, "{}", cell(OLD_GAME, "amml / create"));
    assert_eq!(installed(&old, "@proof/plain").as_deref(), Some("1.1.0"), "{}", cell(OLD_GAME, "plain / defaults"));
    assert_eq!(installed(&old, "@proof/gated").as_deref(), Some("1.0.0"), "{}", cell(OLD_GAME, "gated / defaults"));
    assert_eq!(installed(&old, "@proof/onlygated"), None, "{}", cell(OLD_GAME, "onlygated / defaults"));
    assert_eq!(old.skipped_mods.len(), 1, "{}: {:?}", cell(OLD_GAME, "skipped / defaults"), old.skipped_mods);
    assert_eq!(old.skipped_mods[0].0, "@proof/onlygated", "{}", cell(OLD_GAME, "skipped id / defaults"));
    assert_eq!(
        old.skipped_mods[0].1,
        "it needs AMML 26.62.0",
        "{}: reason",
        cell(OLD_GAME, "skipped reason / defaults")
    );
    assert!(!harness.folder_exists("old", "onlygated"), "{}", cell(OLD_GAME, "onlygated folder / defaults"));

    let new = harness.create(NEW_GAME, "new").unwrap();
    assert_eq!(recorded_amml(&harness, "new"), ProofTree::NEW_AMML, "{}", cell(NEW_GAME, "amml / create"));
    assert_eq!(installed(&new, "@proof/plain").as_deref(), Some("1.1.0"), "{}", cell(NEW_GAME, "plain / defaults"));
    assert_eq!(installed(&new, "@proof/gated").as_deref(), Some("2.0.0"), "{}", cell(NEW_GAME, "gated / defaults"));
    assert_eq!(
        installed(&new, "@proof/onlygated").as_deref(),
        Some("1.0.0"),
        "{}",
        cell(NEW_GAME, "onlygated / defaults")
    );
    assert!(new.skipped_mods.is_empty(), "{}: {:?}", cell(NEW_GAME, "skipped / defaults"), new.skipped_mods);
}

#[test]
fn install_each_mod() {
    let harness = ProofHarness::new();
    assert_tree(&harness);
    let cases = expected_versions();
    for (game, mods) in cases {
        let name = harness.instance_on(game);
        assert_eq!(recorded_amml(&harness, &name), amml_of(game), "{}", cell(game, "amml / before mods"));
        harness.clear_mods(&name);
        for (mod_name, version) in mods {
            let label = cell(game, &format!("{mod_name} / install"));
            let result = harness.install(&name, mod_name);
            match version {
                Some(version) => {
                    assert_eq!(result.unwrap(), ModChange::Installed, "{label}");
                    let state = InstanceReader::read(&harness.paths.instance(&name)).unwrap();
                    let found = state.mods.iter().find(|m| m.id.as_str() == format!("@proof/{mod_name}")).unwrap();
                    assert_eq!(found.version.as_deref(), Some(version), "{label}");
                    assert_eq!(harness.version_on_disk(&name, mod_name).as_deref(), Some(version), "{label}: amml.hcl");
                    assert_eq!(
                        recorded(&harness, &name, mod_name).as_deref(),
                        Some(version),
                        "{label}: launcher record"
                    );
                }
                None => {
                    match result {
                        Err(LauncherError::NeedsAmml { id, needs: found }) => {
                            assert_eq!(id, "@proof/onlygated", "{label}: id");
                            assert_eq!(found, needs("26.62.0", Some(NEW_GAME)), "{label}: needs");
                        }
                        other => panic!("{label}: expected NeedsAmml, got {other:?}"),
                    }
                    assert!(!harness.folder_exists(&name, mod_name), "{label}: no folder");
                }
            }
        }
    }
}

#[test]
fn installed_version_on_disk() {
    let harness = ProofHarness::new();
    assert_tree(&harness);
    let cases = expected_versions();
    for (game, mods) in cases {
        let name = harness.instance_on(game);
        let state = InstanceReader::read(&harness.paths.instance(&name)).unwrap();
        let block = LauncherBlockStore::read(&state.folder).unwrap();
        for (mod_name, version) in mods {
            let label = cell(game, &format!("{mod_name} / on disk"));
            assert_eq!(harness.version_on_disk(&name, mod_name).as_deref(), version, "{label}: amml.hcl");
            let id = format!("@proof/{mod_name}");
            let recorded = block.mods.get(&id).and_then(|entry| entry.version.clone());
            assert_eq!(recorded.as_deref(), version, "{label}: launcher record");
            let read = state.mods.iter().find(|m| m.id.as_str() == id).and_then(|m| m.version.clone());
            assert_eq!(read.as_deref(), version, "{label}: instance reader");
        }
    }
}

type Expected = [(&'static str, Option<&'static str>); 3];

// the version each mod resolves to on a fresh instance of each game
fn expected_versions() -> [(&'static str, Expected); 2] {
    [
        (OLD_GAME, [("plain", Some("1.1.0")), ("gated", Some("1.0.0")), ("onlygated", None)]),
        (NEW_GAME, [("plain", Some("1.1.0")), ("gated", Some("2.0.0")), ("onlygated", Some("1.0.0"))]),
    ]
}

fn recorded(harness: &ProofHarness, name: &str, mod_name: &str) -> Option<String> {
    let block = LauncherBlockStore::read(&harness.paths.instance(name)).unwrap();
    block.mods.get(&format!("@proof/{mod_name}")).and_then(|entry| entry.version.clone())
}

fn pinned(harness: &ProofHarness, game: &str) -> String {
    let name = harness.instance_on(game);
    assert_eq!(recorded_amml(harness, &name), amml_of(game), "{}", cell(game, "amml / before pins"));
    harness.pin(&name, "plain", "1.0.0");
    harness.pin(&name, "gated", "1.0.0");
    for mod_name in ["plain", "gated"] {
        let label = cell(game, &format!("{mod_name} 1.0.0 / pinned"));
        assert_eq!(harness.version_on_disk(&name, mod_name).as_deref(), Some("1.0.0"), "{label}: disk");
        assert_eq!(recorded(harness, &name, mod_name).as_deref(), Some("1.0.0"), "{label}: record");
    }
    name
}

#[test]
fn update_check_rows() {
    let harness = ProofHarness::new();
    assert_tree(&harness);
    let old = pinned(&harness, OLD_GAME);
    let new = pinned(&harness, NEW_GAME);

    let rows = harness.check(&old);
    let gated = row(&rows, "@proof/gated");
    let label = cell(OLD_GAME, "gated 1.0.0 / update check");
    assert_eq!(gated.status, UpdateStatus::NeedsAmml, "{label}: status");
    assert_eq!(gated.available, None, "{label}: available");
    assert_eq!(gated.installed.as_deref(), Some("1.0.0"), "{label}: installed");
    assert_eq!(gated.needs_amml, Some(needs("26.60.6", None)), "{label}: needs");
    let plain = row(&rows, "@proof/plain");
    let label = cell(OLD_GAME, "plain 1.0.0 / update check");
    assert_eq!(plain.status, UpdateStatus::Available, "{label}: status");
    assert_eq!(plain.available.as_deref(), Some("1.1.0"), "{label}: available");

    let rows = harness.check(&new);
    let gated = row(&rows, "@proof/gated");
    let label = cell(NEW_GAME, "gated 1.0.0 / update check");
    assert_eq!(gated.status, UpdateStatus::Available, "{label}: status");
    assert_eq!(gated.available.as_deref(), Some("2.0.0"), "{label}: available");
    assert_eq!(gated.needs_amml, None, "{label}: needs");
    let plain = row(&rows, "@proof/plain");
    let label = cell(NEW_GAME, "plain 1.0.0 / update check");
    assert_eq!(plain.status, UpdateStatus::Available, "{label}: status");
    assert_eq!(plain.available.as_deref(), Some("1.1.0"), "{label}: available");
}

#[test]
fn run_update() {
    let harness = ProofHarness::new();
    assert_tree(&harness);
    let old = pinned(&harness, OLD_GAME);
    let new = pinned(&harness, NEW_GAME);

    let changes = harness.update_all(&old);
    let ids: Vec<&str> = changes.iter().map(|(id, _)| id.as_str()).collect();
    assert!(!ids.contains(&"@proof/gated"), "{}: not Available: {ids:?}", cell(OLD_GAME, "gated / update"));
    assert_eq!(
        harness.version_on_disk(&old, "gated").as_deref(),
        Some("1.0.0"),
        "{}",
        cell(OLD_GAME, "gated / update")
    );
    assert_eq!(
        harness.version_on_disk(&old, "plain").as_deref(),
        Some("1.1.0"),
        "{}",
        cell(OLD_GAME, "plain / update")
    );
    assert_eq!(recorded_amml(&harness, &old), ProofTree::OLD_AMML, "{}", cell(OLD_GAME, "amml / after update"));
    assert_eq!(recorded(&harness, &old, "plain").as_deref(), Some("1.1.0"), "{}", cell(OLD_GAME, "plain record"));
    assert_eq!(recorded(&harness, &old, "gated").as_deref(), Some("1.0.0"), "{}", cell(OLD_GAME, "gated record"));
    // the real resolve path an update uses (the recorded source) also lands on 1.0.0, and installing it keeps 1.0.0
    let (resolved_version, change) = harness.resolve_and_install(&old, "gated");
    let label = cell(OLD_GAME, "gated / resolve for update");
    assert_eq!(resolved_version, "1.0.0", "{label}: resolved");
    assert_eq!(change, ModChange::Updated { from: Some("1.0.0".to_string()) }, "{label}: change");
    assert_eq!(harness.version_on_disk(&old, "gated").as_deref(), Some("1.0.0"), "{label}: disk");
    assert_eq!(recorded(&harness, &old, "gated").as_deref(), Some("1.0.0"), "{label}: record");

    let changes = harness.update_all(&new);
    let gated = changes.iter().find(|(id, _)| id == "@proof/gated").map(|(_, change)| change.clone());
    assert_eq!(
        gated,
        Some(ModChange::Updated { from: Some("1.0.0".to_string()) }),
        "{}",
        cell(NEW_GAME, "gated / update")
    );
    assert_eq!(
        harness.version_on_disk(&new, "gated").as_deref(),
        Some("2.0.0"),
        "{}",
        cell(NEW_GAME, "gated / update")
    );
    assert_eq!(
        harness.version_on_disk(&new, "plain").as_deref(),
        Some("1.1.0"),
        "{}",
        cell(NEW_GAME, "plain / update")
    );
    assert_eq!(recorded(&harness, &new, "gated").as_deref(), Some("2.0.0"), "{}", cell(NEW_GAME, "gated record"));
    assert_eq!(recorded(&harness, &new, "plain").as_deref(), Some("1.1.0"), "{}", cell(NEW_GAME, "plain record"));
}

#[test]
fn no_foreign_host() {
    let harness = ProofHarness::new();
    assert_tree(&harness);
    let old = pinned(&harness, OLD_GAME);
    let new = pinned(&harness, NEW_GAME);
    harness.check(&old);
    harness.update_all(&old);
    harness.update_all(&new);

    // only requests after the tree was built count: the registry tool downloaded every zip itself while building it
    let requests = harness.launcher_requests();
    assert!(
        requests.iter().any(|path| path.ends_with("/registry/registry.json")),
        "the launcher never fetched registry.json: {requests:?}"
    );
    for needle in [
        "amml-26.60.5-windows-x64.zip",
        "amml-26.62.0-windows-x64.zip",
        "plain-1.0.0-windows-x64.zip",
        "plain-1.1.0-windows-x64.zip",
        "gated-1.0.0-windows-x64.zip",
        "gated-2.0.0-windows-x64.zip",
        "onlygated-1.0.0-windows-x64.zip",
    ] {
        assert!(
            requests.iter().any(|path| path.ends_with(needle)),
            "{needle} was never fetched by the launcher: {requests:?}"
        );
    }
}
