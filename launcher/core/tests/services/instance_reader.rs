use crate::support::{InstanceFolder, Roots};
use amml_launcher_core::{InstanceReader, LauncherError, ModSource};

#[test]
fn enabled_and_disabled_mods_are_both_listed() {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("instances").join("one");
    std::fs::create_dir_all(&dir).unwrap();
    InstanceFolder::create(&dir, "1.26.60.05")
        .with_amml(Some("26.60.1"))
        .with_mod(true, "alaydriem", "zoom", Some("26.60.1"), 2)
        .with_mod(false, "alaydriem", "peek", Some("26.60.0"), 2);

    let state = InstanceReader::read(&dir).unwrap();
    assert_eq!(state.name, "one");
    assert_eq!(state.game.instance_name(), "1.26.60.05");
    assert!(!state.managed);
    assert_eq!(state.mods.len(), 2);
    let peek = &state.mods[0];
    assert_eq!(
        (peek.id.as_str().as_str(), peek.enabled, peek.version.as_deref()),
        ("@alaydriem/peek", false, Some("26.60.0"))
    );
    let zoom = &state.mods[1];
    assert_eq!((zoom.id.as_str().as_str(), zoom.enabled, zoom.api_version), ("@alaydriem/zoom", true, 2));
    assert_eq!(zoom.source, ModSource::Local);
    assert_eq!(zoom.listing.as_ref().unwrap().title, "ZOOM");
}

#[test]
fn a_source_built_mod_has_no_version() {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("one");
    std::fs::create_dir_all(&dir).unwrap();
    InstanceFolder::create(&dir, "1.26.60.05").with_mod(true, "me", "mine", None, 2);
    let state = InstanceReader::read(&dir).unwrap();
    assert_eq!(state.mods[0].version, None);
}

#[test]
fn an_unreadable_mod_manifest_lists_the_mod_with_no_version_and_no_listing() {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("one");
    std::fs::create_dir_all(&dir).unwrap();
    let folder = InstanceFolder::create(&dir, "1.26.60.05").with_mod(true, "me", "mine", Some("1.0.0"), 2);
    std::fs::write(folder.mod_folder(true, "me", "mine").join("amml.hcl"), "this is { not hcl").unwrap();
    let state = InstanceReader::read(&dir).unwrap();
    assert_eq!(state.mods.len(), 1);
    assert_eq!(
        (state.mods[0].version.clone(), state.mods[0].listing.clone(), state.mods[0].api_version),
        (None, None, 0)
    );
}

#[test]
fn the_launcher_block_gives_the_source_and_managed() {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("one");
    std::fs::create_dir_all(&dir).unwrap();
    InstanceFolder::create(&dir, "1.26.60.05")
        .with_mod(true, "alaydriem", "zoom", Some("26.60.1"), 2)
        .with_version_json(
            "{\"gameVersion\":\"1.26.60.05\",\"type\":\"preview\",\"launcher\":{\"amml\":\"26.60.1\",\"mods\":{\"@alaydriem/zoom\":{\"version\":\"26.60.1\",\"source\":\"https://r.example/\"}}}}",
        );
    let state = InstanceReader::read(&dir).unwrap();
    assert!(state.managed);
    assert!(state.preview);
    assert_eq!(state.mods[0].source, ModSource::Registry("https://r.example/".to_string()));
}

#[test]
fn a_folder_without_version_json_is_not_an_instance() {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("one");
    std::fs::create_dir_all(&dir).unwrap();
    InstanceFolder::create(&dir, "1.26.60.05");
    std::fs::remove_file(dir.join("version.json")).unwrap();
    assert!(matches!(InstanceReader::read(&dir), Err(LauncherError::NotAnInstance(_))));

    std::fs::write(dir.join("version.json"), "{\"gameVersion\":\"1.26.60.05\"}").unwrap();
    std::fs::remove_file(dir.join("Minecraft.Windows.exe.bak")).unwrap();
    assert!(matches!(InstanceReader::read(&dir), Err(LauncherError::NotAnInstance(_))));
}

#[test]
fn staging_folders_are_skipped_by_list() {
    let root = tempfile::tempdir().unwrap();
    let paths = Roots::paths(root.path());
    for name in ["b", "a", ".staging-1"] {
        let dir = paths.instance(name);
        std::fs::create_dir_all(&dir).unwrap();
        InstanceFolder::create(&dir, "1.26.60.05");
    }
    std::fs::create_dir_all(paths.instance("broken")).unwrap();
    let listed = InstanceReader::list(&paths);
    let names: Vec<String> =
        listed.iter().filter_map(|state| state.as_ref().ok()).map(|state| state.name.clone()).collect();
    assert_eq!(names, vec!["a", "b"]);
    assert_eq!(listed.iter().filter(|state| state.is_err()).count(), 1);
}

#[test]
fn amml_version_comes_from_amml_version_txt() {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("one");
    std::fs::create_dir_all(&dir).unwrap();
    InstanceFolder::create(&dir, "1.26.60.05").with_amml(Some("26.60.1"));
    let state = InstanceReader::read(&dir).unwrap();
    assert!(state.amml.installed);
    assert_eq!(state.amml.version.as_deref(), Some("26.60.1"));

    std::fs::remove_file(dir.join("amml").join("amml-version.txt")).unwrap();
    let state = InstanceReader::read(&dir).unwrap();
    assert!(state.amml.installed);
    assert_eq!(state.amml.version, None);

    std::fs::remove_file(dir.join("amml.dll")).unwrap();
    let state = InstanceReader::read(&dir).unwrap();
    assert!(!state.amml.installed);
}

#[test]
fn the_data_dir_comes_from_instance_hcl() {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("one");
    std::fs::create_dir_all(&dir).unwrap();
    InstanceFolder::create(&dir, "1.26.60.05");
    assert_eq!(InstanceReader::read(&dir).unwrap().data_dir, None);
    std::fs::create_dir_all(dir.join("amml")).unwrap();
    std::fs::write(dir.join("amml").join("instance.hcl"), "data_dir = \"D:\\\\mine\"\n").unwrap();
    assert_eq!(InstanceReader::read(&dir).unwrap().data_dir, Some(std::path::PathBuf::from("D:\\mine")));
}

#[test]
fn amml_state_is_local_for_plus_local_version() {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("one");
    std::fs::create_dir_all(&dir).unwrap();
    InstanceFolder::create(&dir, "1.26.62.01").with_amml(Some("26.60.5+local")).with_amml_api("2");

    let state = InstanceReader::read(&dir).unwrap();

    assert!(state.amml.local);
    assert_eq!(state.amml.local_api, Some(2));
    assert_eq!(state.amml.version.as_deref(), Some("26.60.5+local"));
}

#[test]
fn amml_state_is_not_local_for_release_version() {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("one");
    std::fs::create_dir_all(&dir).unwrap();
    InstanceFolder::create(&dir, "1.26.60.05").with_amml(Some("26.60.4")).with_amml_api("9");

    let state = InstanceReader::read(&dir).unwrap();

    assert!(!state.amml.local);
    assert_eq!(state.amml.local_api, None);
}
