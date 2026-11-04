use crate::support::{InstanceFolder, Roots};
use amml_launcher_core::{DataMigrator, LauncherError};
use std::os::windows::fs::OpenOptionsExt;
use std::path::Path;

fn world(base: &Path, relative: &str) {
    let dir = base.join(relative);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("level.dat"), b"level").unwrap();
    std::fs::write(dir.join("levelname.txt"), b"name").unwrap();
}

fn bedrock(base: &Path) {
    world(base, "Users/1/games/com.mojang/minecraftWorlds/w1");
    world(base, "Users/1/games/com.mojang/minecraftWorlds/w2");
    world(base, "Users/Shared/games/com.mojang/minecraftWorlds/w3");
    std::fs::create_dir_all(base.join("Users/1/games/com.mojang/minecraftWorlds/not-a-world")).unwrap();
    std::fs::create_dir_all(base.join("Users/1/games/com.mojang/resource_packs/p")).unwrap();
    std::fs::write(base.join("Users/1/games/com.mojang/resource_packs/p/manifest.json"), b"{}").unwrap();
}

#[test]
fn candidates_count_worlds_and_skip_missing_folders() {
    let root = tempfile::tempdir().unwrap();
    let paths = Roots::paths(root.path());
    let appdata = root.path().join("appdata").join("Minecraft Bedrock");
    bedrock(&appdata);
    let own = paths.instance("one").join("Minecraft Bedrock");
    world(&own, "Users/1/games/com.mojang/minecraftWorlds/only");
    std::fs::create_dir_all(paths.instance("two")).unwrap();
    std::fs::create_dir_all(paths.instance(".staging-1").join("Minecraft Bedrock")).unwrap();

    let found = DataMigrator::new(appdata.clone(), paths.clone()).candidates();
    assert_eq!(found.len(), 2);
    assert_eq!((found[0].label.as_str(), found[0].world_count, &found[0].path), ("appdata", 3, &appdata));
    assert!(found[0].modified.is_some());
    assert_eq!((found[1].label.as_str(), found[1].world_count), ("one", 1));

    let missing = DataMigrator::new(root.path().join("nope"), paths).candidates();
    assert_eq!(missing.len(), 1);
    assert_eq!(missing[0].label, "one");
}

#[test]
fn copy_fills_an_empty_target_and_leaves_the_source_unchanged() {
    let root = tempfile::tempdir().unwrap();
    let paths = Roots::paths(root.path());
    let appdata = root.path().join("appdata").join("Minecraft Bedrock");
    bedrock(&appdata);
    let before = InstanceFolder::hashes(&appdata);
    let data = root.path().join("data");
    // an empty target folder is allowed
    std::fs::create_dir_all(data.join("Minecraft Bedrock")).unwrap();

    let mut seen = 0;
    let copied = DataMigrator::new(appdata.clone(), paths).copy(&appdata, &data, &mut |count| seen = count).unwrap();
    assert_eq!(copied, before.len() as u64);
    assert_eq!(seen, copied);
    assert_eq!(InstanceFolder::hashes(&appdata), before);
    assert_eq!(InstanceFolder::hashes(&data.join("Minecraft Bedrock")), before);
    assert!(InstanceFolder::names(&data).iter().all(|name| !name.starts_with(".staging-")));
}

#[test]
fn a_non_empty_target_is_refused_and_untouched() {
    let root = tempfile::tempdir().unwrap();
    let paths = Roots::paths(root.path());
    let appdata = root.path().join("appdata").join("Minecraft Bedrock");
    bedrock(&appdata);
    let data = root.path().join("data");
    std::fs::create_dir_all(data.join("Minecraft Bedrock")).unwrap();
    std::fs::write(data.join("Minecraft Bedrock").join("mine.txt"), b"mine").unwrap();
    let before = InstanceFolder::hashes(&data);
    let result = DataMigrator::new(appdata.clone(), paths).copy(&appdata, &data, &mut |_| {});
    assert!(matches!(result, Err(LauncherError::TargetNotEmpty(_))));
    assert_eq!(InstanceFolder::hashes(&data), before);
    assert!(InstanceFolder::names(&data).iter().all(|name| !name.starts_with(".staging-")));
}

#[test]
fn a_failed_copy_leaves_no_staging_folder() {
    let root = tempfile::tempdir().unwrap();
    let paths = Roots::paths(root.path());
    let appdata = root.path().join("appdata").join("Minecraft Bedrock");
    bedrock(&appdata);
    let data = root.path().join("data");
    let held = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(appdata.join("Users/1/games/com.mojang/minecraftWorlds/w2/level.dat"))
        .unwrap();
    let result = DataMigrator::new(appdata.clone(), paths).copy(&appdata, &data, &mut |_| {});
    drop(held);
    assert!(matches!(result, Err(LauncherError::Io(_))), "{result:?}");
    assert!(!data.join("Minecraft Bedrock").exists());
    assert!(InstanceFolder::names(&data).iter().all(|name| !name.starts_with(".staging-")));
    assert!(appdata.join("Users/1/games/com.mojang/minecraftWorlds/w2/level.dat").is_file());
}

#[test]
fn a_source_that_holds_the_data_folder_or_sits_inside_it_is_refused() {
    let root = tempfile::tempdir().unwrap();
    let paths = Roots::paths(root.path());
    let data = root.path().join("data");
    std::fs::create_dir_all(data.join("inner")).unwrap();
    std::fs::write(root.path().join("file.txt"), b"x").unwrap();
    let migrator = DataMigrator::new(root.path().join("none"), paths);
    for source in [root.path().to_path_buf(), data.clone(), data.join("inner")] {
        let result = migrator.copy(&source, &data, &mut |_| {});
        assert!(matches!(result, Err(LauncherError::Io(_))), "{result:?}");
    }
    assert!(InstanceFolder::names(&data).iter().all(|name| !name.starts_with(".staging-")));
}
