use std::fs;
use std::path::Path;

use amml_runtime::ModRegistry;
use tempfile::tempdir;

struct Folders;

impl Folders {
    fn write(root: &Path, folder: &str, manifest: &str) {
        let dir = root.join(folder);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("amml.hcl"), manifest).unwrap();
    }

    fn manifest(name: &str) -> String {
        format!(
            "name = \"{name}\"\nentry = \"{name}.dll\"\napi_version = 1\nentries = [\"B.two\", \"A.one\", \"A.one\"]\n"
        )
    }
}

#[test]
fn mods_load_in_sorted_order_and_a_bad_manifest_skips_only_that_mod() {
    let root = tempdir().unwrap();
    Folders::write(root.path(), "charlie", &Folders::manifest("charlie"));
    Folders::write(root.path(), "alpha", &Folders::manifest("alpha"));
    Folders::write(root.path(), "bravo", "name = \"bravo\"\nentry = \"..\\\\evil.dll\"\napi_version = 1\n");
    Folders::write(root.path(), "delta", "this is { not hcl");
    Folders::write(root.path(), "echo", &Folders::manifest("echo").replace("api_version = 1", "api_version = 0"));
    fs::create_dir_all(root.path().join("no-manifest")).unwrap();
    fs::write(root.path().join("stray.txt"), "x").unwrap();

    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(root.path()).unwrap();

    let names: Vec<&str> = scan.mods.iter().map(|spec| spec.name.as_str()).collect();
    assert_eq!(names, ["alpha", "charlie"]);
    let skipped: Vec<String> =
        scan.skipped.iter().map(|skip| skip.dir.file_name().unwrap().to_string_lossy().into_owned()).collect();
    assert_eq!(skipped, ["bravo", "delta", "echo"]);
    let entries: Vec<&str> = scan.mods[0].entries.iter().map(|id| id.0.as_str()).collect();
    assert_eq!(entries, ["A.one", "B.two"]);
    assert_eq!(scan.mods[0].library_path(), root.path().join("alpha").join("alpha.dll"));
}

#[test]
fn only_the_first_valid_mods_up_to_the_limit_are_accepted() {
    let root = tempdir().unwrap();
    for name in ["a", "b", "c"] {
        Folders::write(root.path(), name, &Folders::manifest(name));
    }

    let scan = ModRegistry::new(2).scan(root.path()).unwrap();

    assert_eq!(scan.mods.len(), 2);
    assert_eq!(scan.skipped.len(), 1);
    assert!(scan.skipped[0].dir.ends_with("c"));
}

#[test]
fn a_second_mod_with_the_same_name_is_skipped() {
    let root = tempdir().unwrap();
    Folders::write(root.path(), "one", &Folders::manifest("same"));
    Folders::write(root.path(), "two", &Folders::manifest("same"));

    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(root.path()).unwrap();

    assert_eq!(scan.mods.len(), 1);
    assert!(scan.skipped[0].dir.ends_with("two"));
}

#[test]
fn a_missing_root_is_an_empty_scan() {
    let root = tempdir().unwrap();

    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&root.path().join("absent")).unwrap();

    assert!(scan.mods.is_empty() && scan.skipped.is_empty());
}

#[test]
fn the_name_loader_is_reserved_in_any_case() {
    let root = tempdir().unwrap();
    Folders::write(root.path(), "one", &Folders::manifest("Loader"));
    Folders::write(root.path(), "two", &Folders::manifest("fine"));

    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(root.path()).unwrap();

    assert_eq!(scan.mods.len(), 1);
    assert_eq!(scan.mods[0].name, "fine");
    assert!(scan.skipped[0].dir.ends_with("one"));
}

#[test]
fn a_name_that_differs_only_by_case_is_a_duplicate() {
    let root = tempdir().unwrap();
    Folders::write(root.path(), "one", &Folders::manifest("Echo"));
    Folders::write(root.path(), "two", &Folders::manifest("echo"));

    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(root.path()).unwrap();

    assert_eq!(scan.mods.len(), 1);
    assert!(scan.skipped[0].dir.ends_with("two"));
}
