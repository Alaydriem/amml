use crate::support::{InstanceFolder, Roots};
use amml_launcher_core::{CopyMode, CopyPlan, DataMigrator, LauncherError};
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

fn run(
    migrator: &DataMigrator,
    source: &Path,
    data: &Path,
    progress: &mut impl FnMut(u64, u64),
) -> Result<(CopyPlan, u64), LauncherError> {
    let mode = migrator.check(source, data)?;
    let plan = migrator.measure(source, data, mode)?;
    let done = migrator.copy(&plan, progress)?;
    Ok((plan, done))
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
fn a_missing_or_empty_target_is_fresh_and_a_target_with_files_is_merge() {
    let root = tempfile::tempdir().unwrap();
    let paths = Roots::paths(root.path());
    let appdata = root.path().join("appdata").join("Minecraft Bedrock");
    bedrock(&appdata);
    let data = root.path().join("data");
    let migrator = DataMigrator::new(appdata.clone(), paths);
    assert_eq!(migrator.check(&appdata, &data).unwrap(), CopyMode::Fresh);
    std::fs::create_dir_all(data.join("Minecraft Bedrock")).unwrap();
    assert_eq!(migrator.check(&appdata, &data).unwrap(), CopyMode::Fresh);
    std::fs::write(data.join("Minecraft Bedrock").join("mine.txt"), b"mine").unwrap();
    assert_eq!(migrator.check(&appdata, &data).unwrap(), CopyMode::Merge);
}

#[test]
fn fresh_copy_reports_bytes_and_leaves_no_staging_folder() {
    let root = tempfile::tempdir().unwrap();
    let paths = Roots::paths(root.path());
    let appdata = root.path().join("appdata").join("Minecraft Bedrock");
    bedrock(&appdata);
    let before = InstanceFolder::hashes(&appdata);
    let data = root.path().join("data");
    // an empty target folder is allowed
    std::fs::create_dir_all(data.join("Minecraft Bedrock")).unwrap();

    let mut last = (0, 0);
    let migrator = DataMigrator::new(appdata.clone(), paths);
    let (plan, done) = run(&migrator, &appdata, &data, &mut |bytes, total| last = (bytes, total)).unwrap();
    assert_eq!(plan.mode, CopyMode::Fresh);
    assert_eq!(plan.files.len(), before.len());
    assert_eq!(plan.total_bytes, plan.files.iter().map(|(_, size)| size).sum::<u64>());
    assert!(plan.total_bytes > 0);
    assert_eq!(done, plan.total_bytes);
    assert_eq!(last, (plan.total_bytes, plan.total_bytes));
    assert_eq!(InstanceFolder::hashes(&appdata), before);
    assert_eq!(InstanceFolder::hashes(&data.join("Minecraft Bedrock")), before);
    assert!(InstanceFolder::names(&data).iter().all(|name| !name.starts_with(".staging-")));
}

#[test]
fn merge_keeps_existing_files_and_copies_only_missing_ones() {
    let root = tempfile::tempdir().unwrap();
    let paths = Roots::paths(root.path());
    let source = root.path().join("appdata").join("Minecraft Bedrock");
    std::fs::create_dir_all(&source).unwrap();
    std::fs::write(source.join("a.txt"), b"theirs").unwrap();
    std::fs::write(source.join("b.txt"), b"bee").unwrap();
    let data = root.path().join("data");
    let target = data.join("Minecraft Bedrock");
    std::fs::create_dir_all(&target).unwrap();
    std::fs::write(target.join("a.txt"), b"mine").unwrap();

    let migrator = DataMigrator::new(source.clone(), paths);
    let (plan, done) = run(&migrator, &source, &data, &mut |_, _| {}).unwrap();
    assert_eq!(plan.mode, CopyMode::Merge);
    assert_eq!((plan.total_bytes, done), (3, 3));
    assert_eq!(std::fs::read(target.join("a.txt")).unwrap(), b"mine");
    assert_eq!(std::fs::read(target.join("b.txt")).unwrap(), b"bee");
}

#[test]
fn merge_retry_overwrites_a_leftover_part_file_and_leaves_none() {
    let root = tempfile::tempdir().unwrap();
    let paths = Roots::paths(root.path());
    let source = root.path().join("appdata").join("Minecraft Bedrock");
    std::fs::create_dir_all(source.join("sub")).unwrap();
    std::fs::write(source.join("b.txt"), b"bee").unwrap();
    std::fs::write(source.join("sub").join("c.txt"), b"see").unwrap();
    let data = root.path().join("data");
    let target = data.join("Minecraft Bedrock");
    std::fs::create_dir_all(target.join("sub")).unwrap();
    std::fs::write(target.join("keep.txt"), b"keep").unwrap();
    std::fs::write(target.join("b.txt.amml-part"), b"half").unwrap();
    std::fs::write(target.join("sub").join("c.txt.amml-part"), b"x").unwrap();
    // a player file that only looks like a part file stays
    std::fs::write(target.join("mine.amml-part"), b"player").unwrap();

    let migrator = DataMigrator::new(source.clone(), paths);
    let (plan, _) = run(&migrator, &source, &data, &mut |_, _| {}).unwrap();
    assert_eq!(plan.total_bytes, 6);
    assert_eq!(std::fs::read(target.join("b.txt")).unwrap(), b"bee");
    assert_eq!(std::fs::read(target.join("sub").join("c.txt")).unwrap(), b"see");
    assert_eq!(std::fs::read(target.join("keep.txt")).unwrap(), b"keep");
    let mut parts: Vec<String> =
        InstanceFolder::names(&target).into_iter().filter(|name| name.ends_with(".amml-part")).collect();
    parts.sort();
    assert_eq!(parts, vec!["mine.amml-part"]);
    assert_eq!(std::fs::read(target.join("mine.amml-part")).unwrap(), b"player");
}

#[test]
fn merge_keeps_a_file_that_appears_after_measure() {
    let root = tempfile::tempdir().unwrap();
    let paths = Roots::paths(root.path());
    let source = root.path().join("appdata").join("Minecraft Bedrock");
    std::fs::create_dir_all(&source).unwrap();
    std::fs::write(source.join("b.txt"), b"theirs").unwrap();
    let data = root.path().join("data");
    let target = data.join("Minecraft Bedrock");
    std::fs::create_dir_all(&target).unwrap();
    std::fs::write(target.join("keep.txt"), b"keep").unwrap();

    let migrator = DataMigrator::new(source.clone(), paths);
    let plan = migrator.measure(&source, &data, CopyMode::Merge).unwrap();
    assert_eq!(plan.files.len(), 1);
    std::fs::write(target.join("b.txt"), b"player").unwrap();
    migrator.copy(&plan, &mut |_, _| {}).unwrap();
    assert_eq!(std::fs::read(target.join("b.txt")).unwrap(), b"player");
    assert!(!target.join("b.txt.amml-part").exists());
}

#[test]
fn a_failed_merge_keeps_player_files_and_leaves_no_part_file() {
    let root = tempfile::tempdir().unwrap();
    let paths = Roots::paths(root.path());
    let source = root.path().join("appdata").join("Minecraft Bedrock");
    std::fs::create_dir_all(&source).unwrap();
    std::fs::write(source.join("a.txt"), b"aaa").unwrap();
    std::fs::write(source.join("locked.txt"), b"lll").unwrap();
    std::fs::write(source.join("z.txt"), b"zzz").unwrap();
    let data = root.path().join("data");
    let target = data.join("Minecraft Bedrock");
    std::fs::create_dir_all(&target).unwrap();
    std::fs::write(target.join("keep.txt"), b"keep").unwrap();
    let held = std::fs::OpenOptions::new().read(true).share_mode(0).open(source.join("locked.txt")).unwrap();

    let result = run(&DataMigrator::new(source.clone(), paths), &source, &data, &mut |_, _| {});
    drop(held);
    assert!(matches!(result, Err(LauncherError::Io(_))), "{result:?}");
    assert_eq!(std::fs::read(target.join("keep.txt")).unwrap(), b"keep");
    assert!(!target.join("locked.txt").exists());
    let names = InstanceFolder::names(&target);
    assert!(names.iter().all(|name| !name.ends_with(".amml-part")), "{names:?}");
}

#[test]
fn undo_after_a_failed_fresh_copy_keeps_files_placed_in_the_target() {
    let root = tempfile::tempdir().unwrap();
    let paths = Roots::paths(root.path());
    let source = root.path().join("appdata").join("Minecraft Bedrock");
    std::fs::create_dir_all(&source).unwrap();
    std::fs::write(source.join("locked.txt"), b"lll").unwrap();
    let data = root.path().join("data");
    let target = data.join("Minecraft Bedrock");
    let migrator = DataMigrator::new(source.clone(), paths);
    let held = std::fs::OpenOptions::new().read(true).share_mode(0).open(source.join("locked.txt")).unwrap();
    let mode = migrator.check(&source, &data).unwrap();
    let plan = migrator.measure(&source, &data, mode).unwrap();
    let result = migrator.copy(&plan, &mut |_, _| {});
    drop(held);
    assert!(result.is_err());

    std::fs::create_dir_all(&target).unwrap();
    std::fs::write(target.join("other.txt"), b"other").unwrap();
    migrator.undo(&plan).unwrap();
    assert_eq!(std::fs::read(target.join("other.txt")).unwrap(), b"other");
}

#[test]
fn an_empty_source_copies_zero_bytes_and_ends() {
    let root = tempfile::tempdir().unwrap();
    let paths = Roots::paths(root.path());
    let source = root.path().join("appdata").join("Minecraft Bedrock");
    std::fs::create_dir_all(&source).unwrap();
    let data = root.path().join("data");
    let mut calls = Vec::new();
    let migrator = DataMigrator::new(source.clone(), paths);
    let (plan, done) = run(&migrator, &source, &data, &mut |bytes, total| calls.push((bytes, total))).unwrap();
    assert_eq!((plan.total_bytes, done), (0, 0));
    assert!(calls.iter().all(|call| *call == (0, 0)));
    assert!(calls.len() <= 1);
}

#[test]
fn undo_removes_a_fresh_target_and_keeps_a_merged_one() {
    let root = tempfile::tempdir().unwrap();
    let paths = Roots::paths(root.path());
    let source = root.path().join("appdata").join("Minecraft Bedrock");
    std::fs::create_dir_all(&source).unwrap();
    std::fs::write(source.join("b.txt"), b"bee").unwrap();
    let data = root.path().join("data");
    let target = data.join("Minecraft Bedrock");
    let migrator = DataMigrator::new(source.clone(), paths);

    let (fresh, _) = run(&migrator, &source, &data, &mut |_, _| {}).unwrap();
    assert!(target.join("b.txt").is_file());
    migrator.undo(&fresh).unwrap();
    assert!(!target.exists());

    std::fs::create_dir_all(&target).unwrap();
    std::fs::write(target.join("mine.txt"), b"mine").unwrap();
    let (merged, _) = run(&migrator, &source, &data, &mut |_, _| {}).unwrap();
    assert_eq!(merged.mode, CopyMode::Merge);
    migrator.undo(&merged).unwrap();
    assert!(target.join("mine.txt").is_file());
    assert!(target.join("b.txt").is_file());
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
    let result = run(&DataMigrator::new(appdata.clone(), paths), &appdata, &data, &mut |_, _| {});
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
        let result = migrator.check(&source, &data);
        assert!(matches!(result, Err(LauncherError::Io(_))), "{result:?}");
    }
    assert!(InstanceFolder::names(&data).iter().all(|name| !name.starts_with(".staging-")));
}
