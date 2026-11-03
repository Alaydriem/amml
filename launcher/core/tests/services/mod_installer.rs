use crate::support::{FakeProcesses, InstanceFolder, Zips};
use amml_launcher_core::{
    InstanceGuard, InstanceReader, LauncherBlockStore, LauncherError, ModChange, ModId, ModInstaller, ModSource,
    VerifiedFile,
};
use std::os::windows::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

struct Setup {
    root: tempfile::TempDir,
    dir: PathBuf,
    processes: FakeProcesses,
}

fn setup() -> Setup {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("one");
    std::fs::create_dir_all(&dir).unwrap();
    InstanceFolder::create(&dir, "1.26.60.05").with_amml(Some("26.60.1"));
    Setup { root, dir, processes: FakeProcesses::new() }
}

fn installer(setup: &Setup) -> ModInstaller<FakeProcesses> {
    ModInstaller::new(InstanceGuard::new(Arc::new(setup.processes.clone())))
}

fn zip(setup: &Setup, version: &str, api: u32, extra: &[(&str, &[u8])]) -> VerifiedFile {
    let path = Zips::mod_zip(&setup.root.path().join(format!("b-{version}-{api}.zip")), "a", "b", version, api, extra);
    VerifiedFile { path, sha256: String::new() }
}

fn registry() -> ModSource {
    ModSource::Registry("https://alaydriem.github.io/amml-registry/".to_string())
}

fn id() -> ModId {
    ModId::parse("@a/b").unwrap()
}

fn leftovers(dir: &Path) -> Vec<String> {
    InstanceFolder::names(dir)
        .into_iter()
        .filter(|name| name.starts_with(".staging-") || name.starts_with(".old-"))
        .collect()
}

#[test]
fn a_new_mod_lands_enabled_with_its_source_recorded() {
    let setup = setup();
    let (installed, change) = installer(&setup)
        .install(&setup.dir, "one", &zip(&setup, "26.60.1", 2, &[]), registry(), 2, None, None)
        .unwrap();
    assert_eq!(installed, id());
    assert_eq!(change, ModChange::Installed);
    let folder = setup.dir.join("mods").join("amml").join("@a").join("b");
    assert!(folder.join("amml.hcl").is_file() && folder.join("b.dll").is_file());
    let block = LauncherBlockStore::read(&setup.dir).unwrap();
    let entry = &block.mods["@a/b"];
    assert_eq!(entry.version.as_deref(), Some("26.60.1"));
    assert_eq!(entry.source, registry());
    assert!(leftovers(&setup.dir).is_empty());
    let state = InstanceReader::read(&setup.dir).unwrap();
    assert!(state.mods[0].enabled);
}

#[test]
fn an_update_keeps_config_and_data_and_replaces_the_rest() {
    let setup = setup();
    let folder = InstanceFolder { dir: setup.dir.clone() }.with_mod(true, "a", "b", Some("26.60.0"), 2);
    let mod_dir = folder.mod_folder(true, "a", "b");
    std::fs::write(mod_dir.join("old.dll"), b"old").unwrap();

    let new_zip = zip(&setup, "26.60.1", 2, &[("new.dll", b"new"), ("config/keys.hcl", b"default keys")]);
    let (_, change) = installer(&setup).install(&setup.dir, "one", &new_zip, registry(), 2, None, None).unwrap();

    assert_eq!(change, ModChange::Updated { from: Some("26.60.0".to_string()) });
    assert_eq!(std::fs::read(mod_dir.join("config").join("keys.hcl")).unwrap(), b"old keys");
    assert_eq!(std::fs::read(mod_dir.join("data").join("db.bin")).unwrap(), b"old db");
    assert!(!mod_dir.join("old.dll").exists());
    assert_eq!(std::fs::read(mod_dir.join("new.dll")).unwrap(), b"new");
    assert_eq!(std::fs::read(mod_dir.join("b.dll")).unwrap(), b"dll 26.60.1");
    assert_eq!(LauncherBlockStore::read(&setup.dir).unwrap().mods["@a/b"].version.as_deref(), Some("26.60.1"));
    assert!(leftovers(&setup.dir).is_empty());
}

#[test]
fn a_failed_update_restores_the_old_folder_exactly() {
    let setup = setup();
    let folder = InstanceFolder { dir: setup.dir.clone() }.with_mod(true, "a", "b", Some("26.60.0"), 2);
    let mod_dir = folder.mod_folder(true, "a", "b");
    let before = InstanceFolder::hashes(&setup.dir.join("mods"));
    let json_before = std::fs::read(setup.dir.join("version.json")).unwrap();

    // an open file inside the mod folder stops the folder from being renamed aside
    let held = std::fs::OpenOptions::new().read(true).share_mode(0).open(mod_dir.join("b.dll")).unwrap();
    let result =
        installer(&setup).install(&setup.dir, "one", &zip(&setup, "26.60.1", 2, &[]), registry(), 2, None, None);
    drop(held);

    assert!(matches!(result, Err(LauncherError::Io(_))), "{result:?}");
    assert_eq!(InstanceFolder::hashes(&setup.dir.join("mods")), before);
    assert_eq!(std::fs::read(setup.dir.join("version.json")).unwrap(), json_before);
    assert!(leftovers(&setup.dir).is_empty());
    assert!(mod_dir.join("config").join("keys.hcl").is_file());
}

#[test]
fn a_bad_zip_changes_nothing() {
    let setup = setup();
    let folder = InstanceFolder { dir: setup.dir.clone() }.with_mod(true, "a", "b", Some("26.60.0"), 2);
    let before = InstanceFolder::hashes(&setup.dir.join("mods"));
    let broken = Zips::raw(&setup.root.path().join("broken.zip"), &[("@a/b/../../x", b"x")]);
    let result = installer(&setup).install(
        &setup.dir,
        "one",
        &VerifiedFile { path: broken, sha256: String::new() },
        registry(),
        2,
        None,
        None,
    );
    assert!(matches!(result, Err(LauncherError::BadArchive { .. })));
    assert_eq!(InstanceFolder::hashes(&setup.dir.join("mods")), before);
    assert!(leftovers(&setup.dir).is_empty());
    assert!(folder.mod_folder(true, "a", "b").is_dir());
}

#[test]
fn an_update_of_a_disabled_mod_stays_disabled() {
    let setup = setup();
    let folder = InstanceFolder { dir: setup.dir.clone() }.with_mod(false, "a", "b", Some("26.60.0"), 2);
    installer(&setup).install(&setup.dir, "one", &zip(&setup, "26.60.1", 2, &[]), registry(), 2, None, None).unwrap();
    let disabled = folder.mod_folder(false, "a", "b");
    assert!(std::fs::read_to_string(disabled.join("amml.hcl")).unwrap().contains("26.60.1"));
    assert_eq!(std::fs::read(disabled.join("config").join("keys.hcl")).unwrap(), b"old keys");
    assert!(!folder.mod_folder(true, "a", "b").exists());
}

#[test]
fn a_mod_for_another_api_is_refused() {
    let setup = setup();
    let before = InstanceFolder::hashes(&setup.dir);
    match installer(&setup).install(&setup.dir, "one", &zip(&setup, "26.60.1", 3, &[]), registry(), 2, None, None) {
        Err(LauncherError::ApiMismatch { id, mod_api, amml_api }) => {
            assert_eq!((id.as_str(), mod_api, amml_api), ("@a/b", 3, 2));
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(InstanceFolder::hashes(&setup.dir), before);
    assert!(!setup.dir.join("mods").exists());
}

#[test]
fn turning_a_mod_off_and_on_moves_the_folder_and_keeps_its_files() {
    let setup = setup();
    let folder = InstanceFolder { dir: setup.dir.clone() }.with_mod(true, "a", "b", Some("26.60.0"), 2);
    let installer = installer(&setup);
    installer.set_enabled(&setup.dir, "one", &id(), false, false).unwrap();
    assert!(!folder.mod_folder(true, "a", "b").exists());
    assert!(!setup.dir.join("mods").join("amml").join("@a").exists());
    assert_eq!(std::fs::read(folder.mod_folder(false, "a", "b").join("config").join("keys.hcl")).unwrap(), b"old keys");

    installer.set_enabled(&setup.dir, "one", &id(), true, true).unwrap();
    assert!(folder.mod_folder(true, "a", "b").join("amml.hcl").is_file());
    assert!(!folder.mod_folder(false, "a", "b").exists());

    let other = ModId::parse("@a/none").unwrap();
    assert!(matches!(
        installer.set_enabled(&setup.dir, "one", &other, false, false),
        Err(LauncherError::UnknownMod(_))
    ));
}

#[test]
fn remove_deletes_the_folder_and_its_launcher_entry() {
    let setup = setup();
    installer(&setup).install(&setup.dir, "one", &zip(&setup, "26.60.1", 2, &[]), registry(), 2, None, None).unwrap();
    installer(&setup).remove(&setup.dir, "one", &id(), false).unwrap();
    assert!(!setup.dir.join("mods").join("amml").join("@a").exists());
    assert!(LauncherBlockStore::read(&setup.dir).unwrap().mods.is_empty());
    assert!(matches!(installer(&setup).remove(&setup.dir, "one", &id(), false), Err(LauncherError::UnknownMod(_))));
}

#[test]
fn core_mods_cannot_be_turned_off_or_removed() {
    let setup = setup();
    let folder = InstanceFolder { dir: setup.dir.clone() }.with_mod(true, "a", "b", Some("26.60.0"), 2);
    let before = InstanceFolder::hashes(&setup.dir);
    let installer = installer(&setup);
    assert!(
        matches!(installer.set_enabled(&setup.dir, "one", &id(), false, true), Err(LauncherError::CoreMod(text)) if text == "@a/b")
    );
    assert!(matches!(installer.remove(&setup.dir, "one", &id(), true), Err(LauncherError::CoreMod(_))));
    assert_eq!(InstanceFolder::hashes(&setup.dir), before);
    assert!(folder.mod_folder(true, "a", "b").is_dir());
}

#[test]
fn a_running_instance_is_refused_before_any_change() {
    let setup = setup();
    InstanceFolder { dir: setup.dir.clone() }.with_mod(true, "a", "b", Some("26.60.0"), 2);
    setup.processes.start(5, setup.dir.join("Minecraft.Windows.exe"));
    let before = InstanceFolder::hashes(&setup.dir);
    let installer = installer(&setup);
    assert!(matches!(
        installer.install(&setup.dir, "one", &zip(&setup, "26.60.1", 2, &[]), registry(), 2, None, None),
        Err(LauncherError::InstanceRunning(_))
    ));
    assert!(matches!(
        installer.set_enabled(&setup.dir, "one", &id(), false, false),
        Err(LauncherError::InstanceRunning(_))
    ));
    assert!(matches!(installer.remove(&setup.dir, "one", &id(), false), Err(LauncherError::InstanceRunning(_))));
    assert_eq!(InstanceFolder::hashes(&setup.dir), before);
    assert!(leftovers(&setup.dir).is_empty());
}

#[test]
fn a_remove_that_fails_leaves_the_mod_whole() {
    let setup = setup();
    let folder = InstanceFolder { dir: setup.dir.clone() }.with_mod(true, "a", "b", Some("26.60.0"), 2);
    let before = InstanceFolder::hashes(&setup.dir.join("mods"));
    let held = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(folder.mod_folder(true, "a", "b").join("b.dll"))
        .unwrap();
    let result = installer(&setup).remove(&setup.dir, "one", &id(), false);
    drop(held);
    assert!(matches!(result, Err(LauncherError::Io(_))), "{result:?}");
    assert_eq!(InstanceFolder::hashes(&setup.dir.join("mods")), before);
    assert!(leftovers(&setup.dir).is_empty());
}

#[test]
fn a_new_mod_whose_parent_folder_cannot_be_made_leaves_nothing() {
    let setup = setup();
    std::fs::create_dir_all(setup.dir.join("mods").join("amml")).unwrap();
    // a file where the namespace folder belongs stops the final move
    std::fs::write(setup.dir.join("mods").join("amml").join("@a"), b"in the way").unwrap();
    let before = InstanceFolder::hashes(&setup.dir);
    let result =
        installer(&setup).install(&setup.dir, "one", &zip(&setup, "26.60.1", 2, &[]), registry(), 2, None, None);
    assert!(matches!(result, Err(LauncherError::Io(_))), "{result:?}");
    assert_eq!(InstanceFolder::hashes(&setup.dir), before);
    assert!(leftovers(&setup.dir).is_empty());
}

#[test]
fn a_launcher_block_that_cannot_be_written_undoes_the_swap() {
    let setup = setup();
    let folder = InstanceFolder { dir: setup.dir.clone() }.with_mod(true, "a", "b", Some("26.60.0"), 2);
    let mod_dir = folder.mod_folder(true, "a", "b");
    // a folder where version.json.tmp belongs makes the block write fail after the swap
    std::fs::create_dir_all(setup.dir.join("version.json.tmp")).unwrap();
    let before = InstanceFolder::hashes(&setup.dir);
    let result =
        installer(&setup).install(&setup.dir, "one", &zip(&setup, "26.60.1", 2, &[]), registry(), 2, None, None);
    assert!(matches!(result, Err(LauncherError::Io(_))), "{result:?}");
    assert_eq!(InstanceFolder::hashes(&setup.dir), before);
    assert!(leftovers(&setup.dir).is_empty());
    assert_eq!(std::fs::read(mod_dir.join("config").join("keys.hcl")).unwrap(), b"old keys");
    assert_eq!(std::fs::read(mod_dir.join("data").join("db.bin")).unwrap(), b"old db");
}

#[test]
fn a_zip_for_another_mod_than_expected_is_refused() {
    let setup = setup();
    let before = InstanceFolder::hashes(&setup.dir);
    let other = ModId::parse("@x/y").unwrap();
    let result = installer(&setup).install(
        &setup.dir,
        "one",
        &zip(&setup, "26.60.1", 2, &[]),
        registry(),
        2,
        Some(&other),
        None,
    );
    assert!(matches!(result, Err(LauncherError::BadArchive { .. })), "{result:?}");
    assert_eq!(InstanceFolder::hashes(&setup.dir), before);
    assert!(!setup.dir.join("mods").exists());
}
