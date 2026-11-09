use crate::support::{FakePatcher, FakeProcesses, InstanceFolder, Zips};
use amml_launcher_core::{AmmlInstaller, CoreProgress, InstanceGuard, LauncherError, VerifiedFile};
use std::os::windows::fs::OpenOptionsExt;
use std::path::Path;
use std::sync::Arc;

struct Setup {
    _root: tempfile::TempDir,
    dir: std::path::PathBuf,
    zip: VerifiedFile,
    processes: FakeProcesses,
}

fn setup() -> Setup {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("one");
    std::fs::create_dir_all(&dir).unwrap();
    let zip = Zips::amml_zip(&root.path().join("amml.zip"), "26.60.1", true);
    Setup { dir, zip: VerifiedFile { path: zip, sha256: String::new() }, processes: FakeProcesses::new(), _root: root }
}

fn installer(setup: &Setup, patcher: FakePatcher) -> AmmlInstaller<FakePatcher, FakeProcesses> {
    AmmlInstaller::new(patcher, InstanceGuard::new(Arc::new(setup.processes.clone())))
}

fn leftovers(dir: &Path) -> Vec<String> {
    InstanceFolder::names(dir).into_iter().filter(|name| name.contains(".old-") || name.contains("staging")).collect()
}

#[test]
fn a_fresh_instance_gets_amml_and_a_patched_exe() {
    let setup = setup();
    InstanceFolder::create(&setup.dir, "1.26.60.05");
    let backup = std::fs::read(setup.dir.join("Minecraft.Windows.exe.bak")).unwrap();
    let patcher = FakePatcher::new();
    let mut steps = Vec::new();
    let version = installer(&setup, patcher.clone())
        .install(&setup.dir, "one", &setup.zip, &mut |step| steps.push(step))
        .unwrap();

    assert_eq!(version, "26.60.1");
    assert!(setup.dir.join("amml.dll").is_file());
    assert!(setup.dir.join("amml").join("recipes").join("a.hcl").is_file());
    assert_eq!(std::fs::read_to_string(setup.dir.join("amml").join("amml-version.txt")).unwrap().trim(), "26.60.1");
    assert!(setup.dir.join("amml").join("cache").is_dir());
    assert!(setup.dir.join("mods").join("amml").is_dir());
    let mut expected = backup.clone();
    expected.extend_from_slice(b"PATCHED");
    assert_eq!(std::fs::read(setup.dir.join("Minecraft.Windows.exe")).unwrap(), expected);
    assert_eq!(std::fs::read(setup.dir.join("Minecraft.Windows.exe.bak")).unwrap(), backup);
    assert!(!setup.dir.join("amml-patch.exe").exists());
    assert!(leftovers(&setup.dir).is_empty());

    let calls = patcher.calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].1, setup.dir.join("Minecraft.Windows.exe.bak"));
    assert!(steps.contains(&CoreProgress::Patching));
}

#[test]
fn an_update_keeps_the_cache_settings_logs_and_mods() {
    let setup = setup();
    let folder = InstanceFolder::create(&setup.dir, "1.26.60.05").with_amml(Some("26.60.0")).with_mod(
        true,
        "a",
        "b",
        Some("1.0.0"),
        2,
    );
    std::fs::create_dir_all(setup.dir.join("amml").join("cache")).unwrap();
    std::fs::create_dir_all(setup.dir.join("amml").join("logs")).unwrap();
    std::fs::write(setup.dir.join("amml").join("cache").join("x"), b"cached").unwrap();
    std::fs::write(setup.dir.join("amml").join("instance.hcl"), b"data_dir = \".\"\n").unwrap();
    std::fs::write(setup.dir.join("amml").join("logs").join("loader.log"), b"log").unwrap();
    let before = InstanceFolder::hashes(&setup.dir);

    installer(&setup, FakePatcher::new()).install(&setup.dir, "one", &setup.zip, &mut |_| {}).unwrap();

    let after = InstanceFolder::hashes(&setup.dir);
    for kept in [
        "amml/cache/x",
        "amml/instance.hcl",
        "amml/logs/loader.log",
        "mods/amml/@a/b/amml.hcl",
        "mods/amml/@a/b/config/keys.hcl",
        "mods/amml/@a/b/data/db.bin",
        "Minecraft.Windows.exe.bak",
        "version.json",
    ] {
        assert_eq!(before.get(kept), after.get(kept), "{kept}");
        assert!(after.contains_key(kept), "{kept}");
    }
    assert!(!after.contains_key("amml/recipes/old.hcl"));
    assert!(after.contains_key("amml/recipes/a.hcl"));
    assert_ne!(before["amml.dll"], after["amml.dll"]);
    assert_eq!(std::fs::read_to_string(setup.dir.join("amml").join("amml-version.txt")).unwrap().trim(), "26.60.1");
    assert!(leftovers(&setup.dir).is_empty());
    assert!(folder.mod_folder(true, "a", "b").is_dir());
}

#[test]
fn a_failed_patch_leaves_the_instance_as_it_was() {
    let setup = setup();
    InstanceFolder::create(&setup.dir, "1.26.60.05").with_amml(Some("26.60.0"));
    let before = InstanceFolder::hashes(&setup.dir);
    let result = installer(&setup, FakePatcher::failing()).install(&setup.dir, "one", &setup.zip, &mut |_| {});
    assert!(matches!(result, Err(LauncherError::PatchFailed { .. })));
    assert_eq!(InstanceFolder::hashes(&setup.dir), before);
    assert!(leftovers(&setup.dir).is_empty());
}

#[test]
fn a_failure_while_placing_restores_every_file() {
    let setup = setup();
    InstanceFolder::create(&setup.dir, "1.26.60.05").with_amml(Some("26.60.0"));
    let before = InstanceFolder::hashes(&setup.dir);
    // an open file inside the recipes folder stops that folder from being renamed, after amml.dll was replaced
    let held = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(setup.dir.join("amml").join("recipes").join("old.hcl"))
        .unwrap();
    let result = installer(&setup, FakePatcher::new()).install(&setup.dir, "one", &setup.zip, &mut |_| {});
    drop(held);
    assert!(matches!(result, Err(LauncherError::Io(_))), "{result:?}");
    assert_eq!(InstanceFolder::hashes(&setup.dir), before);
    assert!(leftovers(&setup.dir).is_empty());
}

#[test]
fn a_running_instance_is_refused_before_any_change() {
    let setup = setup();
    InstanceFolder::create(&setup.dir, "1.26.60.05").with_amml(Some("26.60.0"));
    setup.processes.start(77, setup.dir.join("Minecraft.Windows.exe"));
    let before = InstanceFolder::hashes(&setup.dir);
    let patcher = FakePatcher::new();
    let result = installer(&setup, patcher.clone()).install(&setup.dir, "one", &setup.zip, &mut |_| {});
    assert!(matches!(result, Err(LauncherError::InstanceRunning(_))));
    assert_eq!(InstanceFolder::hashes(&setup.dir), before);
    assert!(patcher.calls().is_empty());
    assert!(leftovers(&setup.dir).is_empty());
}

#[test]
fn a_folder_without_a_backup_is_not_an_instance() {
    let setup = setup();
    std::fs::create_dir_all(&setup.dir).unwrap();
    let result = installer(&setup, FakePatcher::new()).install(&setup.dir, "one", &setup.zip, &mut |_| {});
    assert!(matches!(result, Err(LauncherError::NotAnInstance(_))));
}

fn data_zip(setup: &Setup) -> VerifiedFile {
    let path = Zips::raw(
        &setup._root.path().join("with-data.zip"),
        &[
            ("amml.dll", b"loader"),
            ("amml-patch.exe", b"patcher"),
            ("amml/recipes/a.hcl", b"recipe a"),
            ("amml/data/new.hcl", b"new data"),
            ("amml/amml-version.txt", b"26.60.2"),
        ],
    );
    VerifiedFile { path, sha256: String::new() }
}

#[test]
fn an_archive_with_data_installs_it() {
    let setup = setup();
    InstanceFolder::create(&setup.dir, "1.26.60.05").with_amml(Some("26.60.0"));
    let data = setup.dir.join("amml").join("data");
    std::fs::create_dir_all(&data).unwrap();
    std::fs::write(data.join("old.hcl"), b"old data").unwrap();

    installer(&setup, FakePatcher::new()).install(&setup.dir, "one", &data_zip(&setup), &mut |_| {}).unwrap();

    assert_eq!(std::fs::read(data.join("new.hcl")).unwrap(), b"new data");
    assert!(!data.join("old.hcl").exists());
    assert!(leftovers(&setup.dir).is_empty());
}

#[test]
fn installing_a_release_without_data_removes_old_data() {
    let setup = setup();
    InstanceFolder::create(&setup.dir, "1.26.60.05").with_amml(Some("26.60.0"));
    let data = setup.dir.join("amml").join("data");
    std::fs::create_dir_all(&data).unwrap();
    std::fs::write(data.join("old.hcl"), b"old data").unwrap();

    installer(&setup, FakePatcher::new()).install(&setup.dir, "one", &setup.zip, &mut |_| {}).unwrap();

    assert!(!data.exists());
    assert!(leftovers(&setup.dir).is_empty());
}

#[test]
fn a_failure_after_the_old_data_moved_aside_brings_the_data_back() {
    let setup = setup();
    InstanceFolder::create(&setup.dir, "1.26.60.05").with_amml(Some("26.60.0"));
    let data = setup.dir.join("amml").join("data");
    std::fs::create_dir_all(&data).unwrap();
    std::fs::write(data.join("old.hcl"), b"old data").unwrap();
    let before = InstanceFolder::hashes(&setup.dir);
    // an open file inside the recipes folder stops that folder from being renamed, after the data moved aside
    let held = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(setup.dir.join("amml").join("recipes").join("old.hcl"))
        .unwrap();

    let result = installer(&setup, FakePatcher::new()).install(&setup.dir, "one", &setup.zip, &mut |_| {});

    drop(held);
    assert!(matches!(result, Err(LauncherError::Io(_))), "{result:?}");
    assert_eq!(std::fs::read(data.join("old.hcl")).unwrap(), b"old data");
    assert_eq!(InstanceFolder::hashes(&setup.dir), before);
    assert!(leftovers(&setup.dir).is_empty());
}

#[test]
fn a_failed_patch_keeps_old_data_when_the_release_has_none() {
    let setup = setup();
    InstanceFolder::create(&setup.dir, "1.26.60.05").with_amml(Some("26.60.0"));
    let data = setup.dir.join("amml").join("data");
    std::fs::create_dir_all(&data).unwrap();
    std::fs::write(data.join("old.hcl"), b"old data").unwrap();
    let before = InstanceFolder::hashes(&setup.dir);

    let result = installer(&setup, FakePatcher::failing()).install(&setup.dir, "one", &setup.zip, &mut |_| {});

    assert!(result.is_err());
    assert_eq!(InstanceFolder::hashes(&setup.dir), before);
}
