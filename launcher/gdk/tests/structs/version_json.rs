use amml_gdk::{GameVersion, VersionJson};
use serde_json::Value;

fn written(document: &VersionJson) -> serde_json::Map<String, Value> {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("version.json");
    document.write(&path).unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    match serde_json::from_str::<Value>(&text).unwrap() {
        Value::Object(map) => map,
        other => panic!("not an object: {other}"),
    }
}

#[test]
fn version_json_new_for_writes_the_launcher_base_keys() {
    let game = GameVersion::parse("1.26.62.1").unwrap();

    let map = written(&VersionJson::new_for("test-x", &game, false));

    let keys: Vec<&str> = map.keys().map(String::as_str).collect();
    assert_eq!(
        keys,
        [
            "name",
            "gameVersion",
            "type",
            "packageType",
            "enableIsolation",
            "enableConsole",
            "enableEditorMode",
            "launchArgs",
            "envVars",
            "createdAt"
        ]
    );
    assert_eq!(map["name"], "test-x");
    assert_eq!(map["gameVersion"], "1.26.62.01");
    assert_eq!(map["type"], "release");
    assert_eq!(map["packageType"], "gdk");
    assert_eq!(map["enableIsolation"], false);
    assert_eq!(map["launchArgs"], "");
    assert!(map["createdAt"].as_str().unwrap().ends_with('Z'));
}

#[test]
fn version_json_new_for_a_preview_is_typed_preview() {
    let game = GameVersion::parse("1.26.70.23").unwrap();

    let document = VersionJson::new_for("p", &game, true);

    assert!(document.preview());
    assert_eq!(written(&document)["type"], "preview");
}

#[test]
fn version_json_keeps_unknown_keys() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("version.json");
    std::fs::write(&path, "\u{feff}{\"gameVersion\": \"1.26.60.05\", \"extra\": {\"a\": 1}}").unwrap();

    let document = VersionJson::read(&path).unwrap();
    document.write(&path).unwrap();

    let again = VersionJson::read(&path).unwrap();
    assert_eq!(again.value("extra"), Some(&serde_json::json!({"a": 1})));
    assert_eq!(again.game_version().unwrap(), GameVersion::parse("1.26.60.5").unwrap());
}

#[test]
fn version_json_refuses_a_file_that_is_not_an_object() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("version.json");
    std::fs::write(&path, "[1, 2]").unwrap();

    assert!(VersionJson::read(&path).is_err());
}
