use crate::support::{GameFolder, PeBytes};
use amml_gdk::{AmmlPaths, GameVersion, GdkError, InstanceService, VersionStore};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

struct Rig {
    root: tempfile::TempDir,
    paths: AmmlPaths,
    service: InstanceService,
}

impl Rig {
    fn new() -> Rig {
        let root = tempfile::tempdir().unwrap();
        let paths = AmmlPaths::new(root.path().join("amml"));
        let service = InstanceService::new(paths.clone(), VersionStore::new(paths.clone()));
        Rig { root, paths, service }
    }

    fn version() -> GameVersion {
        GameVersion::parse("1.26.60.5").unwrap()
    }

    // a stored version folder
    fn stored_version(&self) -> PathBuf {
        let dir = self.paths.versions().join("1.26.60.05");
        GameFolder::create(&dir);
        dir
    }

    // every file under `dir` by relative path
    fn snapshot(dir: &Path) -> BTreeMap<String, Vec<u8>> {
        let mut files = BTreeMap::new();
        Self::walk(dir, dir, &mut files);
        files
    }

    fn walk(base: &Path, dir: &Path, files: &mut BTreeMap<String, Vec<u8>>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                Self::walk(base, &entry.path(), files);
            } else {
                let relative = entry.path().strip_prefix(base).unwrap().to_string_lossy().replace('\\', "/");
                files.insert(relative, std::fs::read(entry.path()).unwrap());
            }
        }
    }
}

#[test]
fn create_copies_version_and_writes_bak() {
    let rig = Rig::new();
    let version_dir = rig.stored_version();
    let before = Rig::snapshot(&version_dir);

    let instance = rig.service.create(&Rig::version(), "1.26.60.05-players").unwrap();

    assert_eq!(instance.path, rig.paths.instances().join("1.26.60.05-players"));
    let copied = Rig::snapshot(&instance.path);
    assert_eq!(copied.len(), before.len() + 1);
    for (name, bytes) in &before {
        assert_eq!(copied.get(name), Some(bytes), "{name}");
    }
    assert_eq!(copied.get("Minecraft.Windows.exe.bak"), before.get("Minecraft.Windows.exe"));
    assert_eq!(Rig::snapshot(&version_dir), before);
}

#[test]
fn two_instances_of_one_version_are_independent() {
    let rig = Rig::new();
    let version_dir = rig.stored_version();
    let a = rig.service.create(&Rig::version(), "a").unwrap();
    let b = rig.service.create(&Rig::version(), "b").unwrap();

    std::fs::create_dir_all(a.path.join("mods")).unwrap();
    std::fs::write(a.path.join("mods").join("x"), b"mod").unwrap();

    assert!(!b.path.join("mods").join("x").exists());
    assert!(!version_dir.join("mods").exists());
}

#[test]
fn bad_and_existing_names_are_refused() {
    let rig = Rig::new();
    rig.stored_version();
    for name in ["../x", "a/b", "", ".staging-1", ".", "..", "a b", &"x".repeat(65)] {
        assert!(matches!(rig.service.create(&Rig::version(), name), Err(GdkError::BadInstanceName(_))), "{name:?}");
    }
    rig.service.create(&Rig::version(), "one").unwrap();
    assert!(matches!(rig.service.create(&Rig::version(), "one"), Err(GdkError::InstanceExists(name)) if name == "one"));
}

#[test]
fn a_version_that_is_not_stored_is_unknown() {
    let rig = Rig::new();
    assert!(matches!(rig.service.create(&Rig::version(), "one"), Err(GdkError::UnknownVersion(_))));
    assert!(!rig.paths.instances().join("one").exists());
}

#[test]
fn a_failed_create_leaves_no_staging_folder() {
    let rig = Rig::new();
    let version_dir = rig.stored_version();
    std::fs::remove_file(version_dir.join("Minecraft.Windows.exe")).unwrap();

    assert!(rig.service.create(&Rig::version(), "one").is_err());

    let left: Vec<_> = std::fs::read_dir(rig.paths.instances()).unwrap().collect();
    assert!(left.is_empty());
}

// an instance as a player has it: patched exe, AMML files, a mod folder, logs and the original exe as `.bak`
fn player_instance(dir: &Path) {
    GameFolder::create(dir);
    let mut patched = PeBytes::x64(8192);
    patched.extend_from_slice(b"patched");
    std::fs::write(dir.join("Minecraft.Windows.exe"), patched).unwrap();
    std::fs::write(dir.join("Minecraft.Windows.exe.bak"), PeBytes::x64(8192)).unwrap();
    for folder in ["amml", "mods", "logs", "Minecraft Bedrock"] {
        std::fs::create_dir_all(dir.join(folder)).unwrap();
        std::fs::write(dir.join(folder).join("keep.txt"), folder).unwrap();
    }
    std::fs::write(dir.join("version.json"), b"{\"gameVersion\":\"1.26.60.5\"}").unwrap();
    std::fs::write(dir.join("amml.dll"), b"dll").unwrap();
}

#[test]
fn seed_skips_instance_files_and_uses_bak() {
    let rig = Rig::new();
    let instance = rig.root.path().join("1.26.60.05");
    player_instance(&instance);
    let before = Rig::snapshot(&instance);

    let folder = rig.service.seed_version_from_instance(&instance, &Rig::version()).unwrap();

    let seeded = Rig::snapshot(&folder.path);
    let names: Vec<_> = seeded.keys().cloned().collect();
    assert_eq!(
        names,
        vec!["MicrosoftGame.Config", "Minecraft.Windows.exe", "b.txt", "data/definitions/a.json", "libs/c.dll"]
    );
    assert_eq!(seeded.get("Minecraft.Windows.exe"), before.get("Minecraft.Windows.exe.bak"));
    assert_eq!(folder.path, rig.paths.versions().join("1.26.60.05"));
    assert_eq!(Rig::snapshot(&instance), before);
}

#[test]
fn seed_without_bak_is_refused() {
    let rig = Rig::new();
    let instance = rig.root.path().join("no-bak");
    player_instance(&instance);
    std::fs::remove_file(instance.join("Minecraft.Windows.exe.bak")).unwrap();

    let result = rig.service.seed_version_from_instance(&instance, &Rig::version());

    assert!(matches!(result, Err(GdkError::NotAGame(_))));
    assert!(!rig.paths.versions().join("1.26.60.05").exists());
}

#[test]
fn seed_of_a_stored_version_is_refused_and_leaves_it_alone() {
    let rig = Rig::new();
    let version_dir = rig.stored_version();
    let before = Rig::snapshot(&version_dir);
    let instance = rig.root.path().join("again");
    player_instance(&instance);

    let result = rig.service.seed_version_from_instance(&instance, &Rig::version());

    assert!(matches!(result, Err(GdkError::VersionExists(_))));
    assert_eq!(Rig::snapshot(&version_dir), before);
}

#[test]
fn seed_of_a_folder_without_game_config_leaves_nothing_behind() {
    let rig = Rig::new();
    let instance = rig.root.path().join("broken");
    player_instance(&instance);
    std::fs::remove_file(instance.join("MicrosoftGame.Config")).unwrap();

    let result = rig.service.seed_version_from_instance(&instance, &Rig::version());

    assert!(matches!(result, Err(GdkError::NotAGame(_))));
    let left: Vec<_> = std::fs::read_dir(rig.paths.versions()).unwrap().collect();
    assert!(left.is_empty());
}

#[test]
fn device_names_and_trailing_dots_are_refused() {
    let rig = Rig::new();
    rig.stored_version();
    for name in ["CON", "nul", "Aux.txt", "PRN", "COM1", "lpt9.log", "name."] {
        assert!(matches!(rig.service.create(&Rig::version(), name), Err(GdkError::BadInstanceName(_))), "{name:?}");
    }
    for name in ["COM10", "console", "COM0", "a.b"] {
        assert!(rig.service.create(&Rig::version(), name).is_ok(), "{name:?}");
    }
}

#[test]
fn create_and_seed_wait_for_a_running_install() {
    let rig = Rig::new();
    rig.stored_version();
    let instance = rig.root.path().join("instance");
    player_instance(&instance);
    let _held = amml_gdk::InstallLock::acquire(&rig.paths.lock()).unwrap();

    assert!(matches!(rig.service.create(&Rig::version(), "one"), Err(GdkError::Busy)));
    let other = GameVersion::parse("1.26.52.3").unwrap();
    assert!(matches!(rig.service.seed_version_from_instance(&instance, &other), Err(GdkError::Busy)));
}
