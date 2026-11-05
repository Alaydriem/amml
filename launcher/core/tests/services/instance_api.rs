use crate::support::{InstanceFolder, RegistryTree};
use amml_launcher_core::{InstanceApi, InstanceReader, InstanceState};

fn read(folder: InstanceFolder, dir: &std::path::Path) -> InstanceState {
    let _ = folder;
    InstanceReader::read(dir).unwrap()
}

fn root_with(version: &str, api: u32) -> amml_registry::RegistryRoot {
    RegistryTree::root(RegistryTree::amml(vec![RegistryTree::amml_release(version, api, &["1.26.60.5"])]))
}

#[test]
fn instance_api_reads_api_file_for_local_build() {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("one");
    std::fs::create_dir_all(&dir).unwrap();
    let state =
        read(InstanceFolder::create(&dir, "1.26.62.01").with_amml(Some("26.60.5+local")).with_amml_api("2"), &dir);

    assert_eq!(InstanceApi::of(&state, Some(&root_with("26.60.5", 7))), Some(2));
}

#[test]
fn instance_api_ignores_api_file_for_release() {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("one");
    std::fs::create_dir_all(&dir).unwrap();
    let state = read(InstanceFolder::create(&dir, "1.26.60.05").with_amml(Some("26.60.4")).with_amml_api("9"), &dir);

    assert_eq!(InstanceApi::of(&state, Some(&root_with("26.60.4", 2))), Some(2));
}

#[test]
fn instance_api_falls_back_to_mods_without_api_file() {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("one");
    std::fs::create_dir_all(&dir).unwrap();
    let folder = InstanceFolder::create(&dir, "1.26.62.01").with_amml(Some("26.60.5+local")).with_mod(
        true,
        "a",
        "m",
        Some("1.0.0"),
        2,
    );
    let state = read(folder, &dir);

    assert_eq!(InstanceApi::of(&state, None), Some(2));
}
