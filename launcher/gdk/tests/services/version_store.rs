use crate::support::GameFolder;
use amml_gdk::{AmmlPaths, ExeCopy, GameVersion, GdkError, VersionStore};
use std::path::{Path, PathBuf};

struct Rig {
    root: tempfile::TempDir,
    store: VersionStore,
    source: PathBuf,
    exe: ExeCopy,
}

impl Rig {
    fn new() -> Rig {
        let root = tempfile::tempdir().unwrap();
        let source = root.path().join("package");
        GameFolder::create(&source);
        let exe_dir = root.path().join("exe");
        std::fs::create_dir_all(&exe_dir).unwrap();
        let exe_path = exe_dir.join("Minecraft.Windows.exe");
        std::fs::copy(source.join("Minecraft.Windows.exe"), &exe_path).unwrap();
        let bytes = std::fs::metadata(&exe_path).unwrap().len();
        let store = VersionStore::new(AmmlPaths::new(root.path().join("amml")));
        Rig { root, store, source, exe: ExeCopy { path: exe_path, bytes, method: "plain" } }
    }

    fn versions(&self) -> PathBuf {
        self.root.path().join("amml").join("versions")
    }

    fn version() -> GameVersion {
        GameVersion::parse("1.26.52.3").unwrap()
    }

    fn entries(dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = match std::fs::read_dir(dir) {
            Ok(read) => read.map(|e| e.unwrap().file_name().to_string_lossy().to_string()).collect(),
            Err(_) => Vec::new(),
        };
        names.sort();
        names
    }
}

#[test]
fn copy_then_commit_creates_version_folder() {
    let rig = Rig::new();
    let staging = rig.store.begin().unwrap();
    let copied = rig.store.copy_tree(&rig.source, &staging, true).unwrap();
    assert_eq!(copied, GameFolder::FILES - 1);

    let folder = rig.store.commit(&staging, &Rig::version(), rig.exe.clone(), GameFolder::FILES).unwrap();

    assert_eq!(folder.path, rig.versions().join("1.26.52.03"));
    assert_eq!(folder.files, GameFolder::FILES);
    assert!(folder.path.join("data").join("definitions").join("a.json").is_file());
    assert_eq!(Rig::entries(&rig.versions()), vec!["1.26.52.03"]);
    assert_eq!(rig.store.get(&Rig::version()).unwrap(), folder);
}

#[test]
fn missing_config_fails_and_cleans_staging() {
    let rig = Rig::new();
    std::fs::remove_file(rig.source.join("MicrosoftGame.Config")).unwrap();
    let staging = rig.store.begin().unwrap();
    rig.store.copy_tree(&rig.source, &staging, true).unwrap();

    let result = rig.store.commit(&staging, &Rig::version(), rig.exe.clone(), GameFolder::FILES - 1);

    assert!(matches!(result, Err(GdkError::NotAGame(_))));
    assert!(Rig::entries(&rig.versions()).is_empty());
}

#[test]
fn count_mismatch_is_incomplete_copy() {
    let rig = Rig::new();
    let staging = rig.store.begin().unwrap();
    rig.store.copy_tree(&rig.source, &staging, true).unwrap();

    let result = rig.store.commit(&staging, &Rig::version(), rig.exe.clone(), GameFolder::FILES + 1);

    assert!(matches!(result, Err(GdkError::IncompleteCopy { expected: 6, copied: 5 })));
    assert!(Rig::entries(&rig.versions()).is_empty());
}

#[test]
fn a_non_x64_exe_is_not_a_game() {
    let rig = Rig::new();
    std::fs::write(&rig.exe.path, [0u8; 8192]).unwrap();
    let staging = rig.store.begin().unwrap();
    rig.store.copy_tree(&rig.source, &staging, true).unwrap();

    let result = rig.store.commit(&staging, &Rig::version(), rig.exe.clone(), GameFolder::FILES);

    assert!(matches!(result, Err(GdkError::NotAGame(_))));
    assert!(Rig::entries(&rig.versions()).is_empty());
}

#[test]
fn existing_version_is_refused() {
    let rig = Rig::new();
    let existing = rig.versions().join("1.26.52.03");
    std::fs::create_dir_all(&existing).unwrap();
    std::fs::write(existing.join("keep.txt"), b"keep").unwrap();
    let staging = rig.store.begin().unwrap();
    rig.store.copy_tree(&rig.source, &staging, true).unwrap();

    let result = rig.store.commit(&staging, &Rig::version(), rig.exe.clone(), GameFolder::FILES);

    assert!(matches!(result, Err(GdkError::VersionExists(name)) if name == "1.26.52.03"));
    assert_eq!(std::fs::read(existing.join("keep.txt")).unwrap(), b"keep");
    assert_eq!(Rig::entries(&rig.versions()), vec!["1.26.52.03"]);
}

#[test]
fn clean_stale_removes_only_staging() {
    let rig = Rig::new();
    let instances = rig.root.path().join("amml").join("instances");
    std::fs::create_dir_all(rig.versions().join(".staging-a")).unwrap();
    std::fs::create_dir_all(instances.join(".staging-b")).unwrap();
    std::fs::create_dir_all(instances.join("play")).unwrap();
    std::fs::create_dir_all(rig.versions().join("1.26.60.05")).unwrap();
    let read_only = rig.versions().join(".staging-a").join("locked.txt");
    std::fs::write(&read_only, b"x").unwrap();
    let mut permissions = std::fs::metadata(&read_only).unwrap().permissions();
    permissions.set_readonly(true);
    std::fs::set_permissions(&read_only, permissions).unwrap();

    let removed = rig.store.clean_stale().unwrap();

    assert_eq!(removed.len(), 2);
    assert_eq!(Rig::entries(&rig.versions()), vec!["1.26.60.05"]);
    assert_eq!(Rig::entries(&instances), vec!["play"]);
}

#[test]
fn list_skips_staging_and_orders_newest_first() {
    let rig = Rig::new();
    for name in ["1.26.52.03", "1.26.60.05", ".staging-x", "notes"] {
        let dir = rig.versions().join(name);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("f"), b"x").unwrap();
    }
    let listed: Vec<String> = rig.store.list().unwrap().iter().map(|f| f.version.instance_name()).collect();
    assert_eq!(listed, vec!["1.26.60.05", "1.26.52.03"]);
}

#[test]
fn clean_stale_goes_on_when_one_folder_cannot_be_deleted() {
    use std::os::windows::fs::OpenOptionsExt;
    let rig = Rig::new();
    let held_dir = rig.versions().join(".staging-held");
    std::fs::create_dir_all(&held_dir).unwrap();
    let held_file = held_dir.join("held.bin");
    std::fs::write(&held_file, b"x").unwrap();
    let _held = std::fs::OpenOptions::new().read(true).share_mode(0).open(&held_file).unwrap();
    std::fs::create_dir_all(rig.versions().join(".staging-free")).unwrap();

    let removed = rig.store.clean_stale().unwrap();

    assert_eq!(removed, vec![rig.versions().join(".staging-free")]);
    assert!(held_dir.exists());
}
