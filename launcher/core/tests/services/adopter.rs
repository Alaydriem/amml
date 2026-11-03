use crate::support::{Harness, InstanceFolder};
use amml_gdk::{GameVersion, InstanceService, VersionStore};
use amml_launcher_core::{Adopter, DataChoice, LauncherBlockStore, LauncherError, ModSource};
use std::path::PathBuf;

const HAND_EDITED: &str = "{\n  \"zzz\": {\"keep\": [1, {\"deep\": true}]},\n  \"name\": \"mine\",\n  \"gameVersion\": \"1.26.60.05\",\n  \"type\": \"release\",\n  \"enableIsolation\": true,\n  \"launchArgs\": \"-a\",\n  \"envVars\": \"A=1\",\n  \"aaa\": \"last custom\"\n}\n";

fn services(harness: &Harness) -> (InstanceService, VersionStore) {
    let gdk = harness.paths.gdk().clone();
    (InstanceService::new(gdk.clone(), VersionStore::new(gdk.clone())), VersionStore::new(gdk))
}

fn adoptable(harness: &Harness, name: &str) -> PathBuf {
    let dir = harness.paths.instance(name);
    std::fs::create_dir_all(&dir).unwrap();
    InstanceFolder::create(&dir, "1.26.60.05")
        .with_amml(Some("26.60.1"))
        .with_mod(true, "me", "mine", None, 2)
        .with_mod(false, "alaydriem", "zoom", Some("26.60.0"), 2)
        .with_version_json(HAND_EDITED);
    // a patched exe that differs from the backup
    std::fs::write(dir.join("Minecraft.Windows.exe"), b"patched exe").unwrap();
    dir
}

fn keys(path: &std::path::Path) -> Vec<(String, String)> {
    let value: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    value.as_object().unwrap().iter().map(|(key, value)| (key.clone(), value.to_string())).collect()
}

#[test]
fn adoption_changes_only_version_json_and_adds_instance_hcl() {
    let harness = Harness::new();
    let dir = adoptable(&harness, "mine");
    let before = InstanceFolder::hashes(&dir);
    let keys_before = keys(&dir.join("version.json"));
    let (instances, store) = services(&harness);

    let shared = harness.paths.default_data_dir();
    let state =
        Adopter::adopt(&harness.paths, &instances, &store, "mine", &DataChoice::Shared(shared.clone())).unwrap();

    let after = InstanceFolder::hashes(&dir);
    let changed: Vec<&String> =
        before.iter().filter(|(path, hash)| after.get(*path) != Some(*hash)).map(|(path, _)| path).collect();
    assert_eq!(changed, vec!["version.json"]);
    let added: Vec<&String> = after.keys().filter(|path| !before.contains_key(*path)).collect();
    assert_eq!(added, vec!["amml/instance.hcl"]);
    assert!(state.managed);
    assert_eq!(state.data_dir, Some(shared));

    let keys_after = keys(&dir.join("version.json"));
    let names: Vec<&str> = keys_after.iter().map(|(key, _)| key.as_str()).collect();
    assert_eq!(
        names,
        vec!["zzz", "name", "gameVersion", "type", "enableIsolation", "launchArgs", "envVars", "aaa", "launcher"]
    );
    for (key, value) in &keys_before {
        if key != "enableIsolation" {
            assert!(keys_after.contains(&(key.clone(), value.clone())), "{key}");
        }
    }
    assert!(keys_after.contains(&("enableIsolation".to_string(), "false".to_string())));
}

#[test]
fn a_source_built_mod_is_adopted_with_no_version() {
    let harness = Harness::new();
    let dir = adoptable(&harness, "mine");
    let (instances, store) = services(&harness);
    Adopter::adopt(&harness.paths, &instances, &store, "mine", &DataChoice::Own).unwrap();
    let block = LauncherBlockStore::read(&dir).unwrap();
    assert_eq!(block.amml.as_deref(), Some("26.60.1"));
    assert_eq!(block.mods["@me/mine"].version, None);
    assert_eq!(block.mods["@me/mine"].source, ModSource::Local);
    assert_eq!(block.mods["@alaydriem/zoom"].version.as_deref(), Some("26.60.0"));
    assert_eq!(block.mods["@alaydriem/zoom"].source, ModSource::Local);
}

#[test]
fn an_existing_instance_hcl_is_kept() {
    let harness = Harness::new();
    let dir = adoptable(&harness, "mine");
    std::fs::write(dir.join("amml").join("instance.hcl"), "data_dir = \"D:\\\\mine\"\n").unwrap();
    let (instances, store) = services(&harness);
    let state = Adopter::adopt(&harness.paths, &instances, &store, "mine", &DataChoice::Own).unwrap();
    assert_eq!(std::fs::read_to_string(dir.join("amml").join("instance.hcl")).unwrap(), "data_dir = \"D:\\\\mine\"\n");
    assert_eq!(state.data_dir, Some(PathBuf::from("D:\\mine")));
}

#[test]
fn a_managed_instance_is_not_a_candidate_and_is_not_changed() {
    let harness = Harness::new();
    let managed = adoptable(&harness, "managed");
    LauncherBlockStore::set_amml(&managed, "26.60.1").unwrap();
    adoptable(&harness, "plain");
    let candidates = Adopter::candidates(&harness.paths);
    assert_eq!(candidates.len(), 1);
    let found = &candidates[0];
    assert_eq!(found.name, "plain");
    assert_eq!(found.game, GameVersion::parse("1.26.60.5").unwrap());
    assert_eq!(found.amml.as_deref(), Some("26.60.1"));
    assert_eq!(found.mod_count, 2);
    assert!(!found.has_version_in_store);

    let before = InstanceFolder::hashes(&managed);
    let (instances, store) = services(&harness);
    Adopter::adopt(&harness.paths, &instances, &store, "managed", &DataChoice::Own).unwrap();
    assert_eq!(InstanceFolder::hashes(&managed), before);
}

#[test]
fn a_missing_version_is_seeded_from_the_bak() {
    let harness = Harness::new();
    let dir = adoptable(&harness, "mine");
    let (instances, store) = services(&harness);
    let game = GameVersion::parse("1.26.60.5").unwrap();
    assert!(store.get(&game).is_none());
    Adopter::adopt(&harness.paths, &instances, &store, "mine", &DataChoice::Own).unwrap();

    let folder = store.get(&game).unwrap();
    let backup = std::fs::read(dir.join("Minecraft.Windows.exe.bak")).unwrap();
    assert_eq!(std::fs::read(folder.path.join("Minecraft.Windows.exe")).unwrap(), backup);
    assert!(
        !folder.path.join("mods").exists()
            && !folder.path.join("amml").exists()
            && !folder.path.join("amml.dll").exists()
    );
    assert!(Adopter::candidates(&harness.paths).is_empty());
}

#[test]
fn a_seed_failure_does_not_stop_the_adoption() {
    let harness = Harness::new();
    let dir = adoptable(&harness, "mine");
    // without the game config the copy is not a complete game folder
    std::fs::remove_file(dir.join("MicrosoftGame.Config")).unwrap();
    let (instances, store) = services(&harness);
    let state = Adopter::adopt(&harness.paths, &instances, &store, "mine", &DataChoice::Own).unwrap();
    assert!(state.managed);
    assert!(store.get(&GameVersion::parse("1.26.60.5").unwrap()).is_none());
}

#[test]
fn a_name_that_is_not_an_instance_is_refused() {
    let harness = Harness::new();
    let (instances, store) = services(&harness);
    assert!(matches!(
        Adopter::adopt(&harness.paths, &instances, &store, "missing", &DataChoice::Own),
        Err(LauncherError::NotAnInstance(_))
    ));
    for bad in ["..", "a\\b", ".hidden"] {
        assert!(
            matches!(
                Adopter::adopt(&harness.paths, &instances, &store, bad, &DataChoice::Own),
                Err(LauncherError::Gdk(_))
            ),
            "{bad}"
        );
    }
}

#[test]
fn core_mods_are_added_to_an_adopted_instance_once() {
    let harness = Harness::new();
    let amml = harness.amml_release("26.60.1", 2, &["1.26.60.5"]);
    let docs = [
        harness.mod_document("amml", "modmanager", true, false, "26.60.1", 2),
        harness.mod_document("alaydriem", "zoom", false, true, "26.60.1", 2),
    ];
    harness.publish(vec![amml], &docs);
    let dir = adoptable(&harness, "mine");
    let (instances, store) = services(&harness);
    Adopter::adopt(&harness.paths, &instances, &store, "mine", &DataChoice::Own).unwrap();

    let mut manager = harness.manager();
    let installed = manager.ensure_core_mods("mine", &mut |_| {}).unwrap();
    assert_eq!(installed, vec!["@amml/modmanager"]);
    assert!(dir.join("mods").join("amml").join("@amml").join("modmanager").join("amml.hcl").is_file());
    // the default mod zoom is not a core mod, so it is not added
    assert!(dir.join("mods").join("amml-disabled").join("@alaydriem").join("zoom").is_dir());
    assert!(!dir.join("mods").join("amml").join("@alaydriem").exists());
    assert!(manager.ensure_core_mods("mine", &mut |_| {}).unwrap().is_empty());
    assert!(harness.downloads_empty());
}
