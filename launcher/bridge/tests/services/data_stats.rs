use amml_launcher_bridge::DataStats;
use std::path::Path;

fn write(path: &Path, size: usize) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, vec![0u8; size]).unwrap();
}

#[test]
fn data_stats_count_worlds_and_packs_in_every_user_folder() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let games = |user: &str| root.join("Users").join(user).join("games").join("com.mojang");
    write(&games("one").join("minecraftWorlds").join("w1").join("level.dat"), 10);
    write(&games("one").join("minecraftWorlds").join("w2").join("level.dat"), 10);
    write(&games("one").join("minecraftWorlds").join("w2").join("db").join("000001.ldb"), 100);
    write(&games("two").join("minecraftWorlds").join("w3").join("level.dat"), 10);
    write(&games("two").join("resource_packs").join("rp1").join("manifest.json"), 5);
    write(&games("two").join("behavior_packs").join("bp1").join("manifest.json"), 5);
    write(&games("two").join("behavior_packs").join("bp2").join("manifest.json"), 5);
    write(&root.join("Shared").join("resource_packs").join("rp2").join("manifest.json"), 5);

    let figures = DataStats::of(root);

    assert_eq!(figures.worlds, 3);
    assert_eq!(figures.resource_packs, 2);
    assert_eq!(figures.behavior_packs, 2);
    assert_eq!(figures.bytes, 10 + 10 + 100 + 10 + 5 + 5 + 5 + 5);
}

#[test]
fn a_world_folder_without_a_level_file_is_not_a_world() {
    let dir = tempfile::tempdir().unwrap();
    write(&dir.path().join("minecraftWorlds").join("broken").join("db").join("a.ldb"), 1);
    write(&dir.path().join("minecraftWorlds").join("loose.txt"), 1);

    let figures = DataStats::of(dir.path());

    assert_eq!(figures.worlds, 0);
    assert_eq!(figures.bytes, 2);
}

#[test]
fn a_missing_folder_is_empty() {
    let dir = tempfile::tempdir().unwrap();
    let figures = DataStats::of(&dir.path().join("none"));
    assert_eq!((figures.worlds, figures.resource_packs, figures.behavior_packs, figures.bytes), (0, 0, 0, 0));
}
