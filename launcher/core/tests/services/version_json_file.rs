use amml_gdk::GameVersion;
use amml_launcher_core::{LauncherBlock, LauncherModEntry, ModSource, VersionJsonFile};
use std::collections::BTreeMap;

const HAND_EDITED: &str = "{\n  \"zzz\": {\"keep\": [1, 2, {\"deep\": true}]},\n  \"name\": \"x\",\n  \"gameVersion\": \"1.26.60.05\",\n  \"enableIsolation\": true,\n  \"launchArgs\": \"-a \\\"b c\\\"\",\n  \"envVars\": \"A=1\",\n  \"aaa\": \"last custom\"\n}\n";

fn keys(path: &std::path::Path) -> Vec<(String, String)> {
    let value: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    value.as_object().unwrap().iter().map(|(key, value)| (key.clone(), value.to_string())).collect()
}

#[test]
fn unknown_keys_and_their_order_survive_a_rewrite() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("version.json");
    std::fs::write(&path, HAND_EDITED).unwrap();
    let before = keys(&path);

    let mut file = VersionJsonFile::read(&path).unwrap();
    file.set_isolation(false);
    let block = LauncherBlock::new(Some("26.60.1".to_string()), BTreeMap::new());
    file.set_launcher(&block);
    file.write(&path).unwrap();

    let after = keys(&path);
    let names: Vec<&str> = after.iter().map(|(key, _)| key.as_str()).collect();
    assert_eq!(
        names,
        vec!["zzz", "name", "gameVersion", "enableIsolation", "launchArgs", "envVars", "aaa", "launcher"]
    );
    for (key, value) in &before {
        if key != "enableIsolation" {
            assert!(after.contains(&(key.clone(), value.clone())), "{key}");
        }
    }
    assert!(after.contains(&("enableIsolation".to_string(), "false".to_string())));
    assert!(!dir.path().join("version.json.tmp").exists());

    let reread = VersionJsonFile::read(&path).unwrap();
    assert_eq!(reread.launch_args(), "-a \"b c\"");
    assert_eq!(reread.env_vars(), "A=1");
    assert!(!reread.isolation());
}

#[test]
fn the_launcher_block_reads_back_with_sources() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("version.json");
    std::fs::write(&path, HAND_EDITED).unwrap();
    let mut mods = BTreeMap::new();
    mods.insert(
        "@alaydriem/zoom".to_string(),
        LauncherModEntry {
            version: Some("26.60.1".to_string()),
            source: ModSource::Registry("https://alaydriem.github.io/amml-registry/".to_string()),
            core: Some(false),
        },
    );
    mods.insert(
        "@x/y".to_string(),
        LauncherModEntry { version: None, source: ModSource::Url("https://x/y.zip".to_string()), core: None },
    );
    mods.insert(
        "@x/z".to_string(),
        LauncherModEntry { version: Some("1.0.0".to_string()), source: ModSource::Local, core: Some(true) },
    );
    let block = LauncherBlock::new(Some("26.60.1".to_string()), mods);

    let mut file = VersionJsonFile::read(&path).unwrap();
    assert!(file.launcher().is_none());
    file.set_launcher(&block);
    file.write(&path).unwrap();

    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("\"source\": \"url:https://x/y.zip\""), "{text}");
    assert!(text.contains("\"source\": \"local\""), "{text}");
    assert_eq!(VersionJsonFile::read(&path).unwrap().launcher().unwrap(), block);
}

#[test]
fn a_new_file_has_isolation_off_and_the_instance_form_of_the_version() {
    let game = GameVersion::parse("1.26.60.5").unwrap();
    let file = VersionJsonFile::new_for("test", &game, false);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("version.json");
    file.write(&path).unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("\"gameVersion\": \"1.26.60.05\""), "{text}");
    assert!(text.contains("\"enableIsolation\": false"), "{text}");
    assert!(text.contains("\"type\": \"release\""), "{text}");
    assert!(text.contains("\"packageType\": \"gdk\""), "{text}");
    let reread = VersionJsonFile::read(&path).unwrap();
    assert_eq!(reread.game_version().unwrap(), game);
    assert!(!reread.preview());
    assert!(VersionJsonFile::new_for("p", &game, true).preview());
}

#[test]
fn a_byte_order_mark_and_a_non_object_are_handled() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("version.json");
    std::fs::write(&path, format!("\u{feff}{HAND_EDITED}")).unwrap();
    assert!(VersionJsonFile::read(&path).is_ok());
    std::fs::write(&path, "[1]").unwrap();
    assert!(matches!(VersionJsonFile::read(&path), Err(amml_launcher_core::LauncherError::BadVersionJson(_))));
}
