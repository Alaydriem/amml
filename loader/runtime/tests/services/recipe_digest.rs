use amml_derive::DERIVER_VERSION;
use amml_runtime::RecipeDigest;

fn write(dir: &std::path::Path, name: &str, text: &str) {
    let path = dir.join(name);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

#[test]
fn the_digest_follows_file_contents_and_paths_but_not_the_directory() {
    let (a, b) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    for dir in [a.path(), b.path()] {
        write(dir, "one.hcl", "x");
        write(dir, "sub/two.hcl", "y");
    }
    let same = RecipeDigest::of_dir(a.path()).unwrap();
    assert_eq!(same, RecipeDigest::of_dir(b.path()).unwrap());
    assert_eq!(same.len(), 16);

    write(b.path(), "sub/two.hcl", "z");
    assert_ne!(same, RecipeDigest::of_dir(b.path()).unwrap());

    write(b.path(), "sub/two.hcl", "y");
    std::fs::rename(b.path().join("sub/two.hcl"), b.path().join("sub/renamed.hcl")).unwrap();
    assert_ne!(same, RecipeDigest::of_dir(b.path()).unwrap());
}

#[test]
fn files_that_are_not_recipes_do_not_change_the_digest() {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "one.hcl", "x");
    let before = RecipeDigest::of_dir(dir.path()).unwrap();
    write(dir.path(), "notes.txt", "anything");
    assert_eq!(before, RecipeDigest::of_dir(dir.path()).unwrap());
}

#[test]
fn another_deriver_version_gives_another_digest_for_the_same_recipes() {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "one.hcl", "x");

    let current = RecipeDigest::of_dir_with(dir.path(), DERIVER_VERSION).unwrap();

    assert_eq!(current, RecipeDigest::of_dir(dir.path()).unwrap());
    assert_ne!(current, RecipeDigest::of_dir_with(dir.path(), DERIVER_VERSION + 1).unwrap());
}
