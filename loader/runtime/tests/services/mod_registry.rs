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

    fn manifest(namespace: &str, name: &str) -> String {
        format!(
            "namespace = \"{namespace}\"\nname = \"{name}\"\nentry = \"{name}.dll\"\napi_version = 2\nentries = [\"B.two\", \"A.one\", \"A.one\"]\n"
        )
    }
}

#[test]
fn mods_load_in_sorted_order_and_a_bad_manifest_skips_only_that_mod() {
    let root = tempdir().unwrap();
    Folders::write(root.path(), "@ns/charlie", &Folders::manifest("ns", "charlie"));
    Folders::write(root.path(), "@ns/alpha", &Folders::manifest("ns", "alpha"));
    Folders::write(
        root.path(),
        "@ns/bravo",
        "namespace = \"ns\"\nname = \"bravo\"\nentry = \"..\\\\evil.dll\"\napi_version = 2\n",
    );
    Folders::write(root.path(), "@ns/delta", "this is { not hcl");
    Folders::write(
        root.path(),
        "@ns/echo",
        &Folders::manifest("ns", "echo").replace("api_version = 2", "api_version = 0"),
    );
    fs::create_dir_all(root.path().join("@ns/no-manifest")).unwrap();
    fs::write(root.path().join("stray.txt"), "x").unwrap();

    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(root.path()).unwrap();

    let ids: Vec<String> = scan.mods.iter().map(|spec| spec.id()).collect();
    assert_eq!(ids, ["@ns/alpha", "@ns/charlie"]);
    let skipped: Vec<String> =
        scan.skipped.iter().map(|skip| skip.dir.file_name().unwrap().to_string_lossy().into_owned()).collect();
    assert_eq!(skipped, ["bravo", "delta", "echo"]);
    let entries: Vec<&str> = scan.mods[0].entries.iter().map(|id| id.0.as_str()).collect();
    assert_eq!(entries, ["A.one", "B.two"]);
    assert_eq!(scan.mods[0].library_path(), root.path().join("@ns").join("alpha").join("alpha.dll"));
}

#[test]
fn only_the_first_valid_mods_up_to_the_limit_are_accepted() {
    let root = tempdir().unwrap();
    for name in ["a", "b", "c"] {
        Folders::write(root.path(), &format!("@ns/{name}"), &Folders::manifest("ns", name));
    }

    let scan = ModRegistry::new(2).scan(root.path()).unwrap();

    assert_eq!(scan.mods.len(), 2);
    assert_eq!(scan.skipped.len(), 1);
    assert!(scan.skipped[0].dir.ends_with("c"));
}

#[test]
fn a_missing_root_is_an_empty_scan() {
    let root = tempdir().unwrap();

    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&root.path().join("absent")).unwrap();

    assert!(scan.mods.is_empty() && scan.skipped.is_empty());
}

#[test]
fn amml_loader_is_reserved_in_any_case() {
    let root = tempdir().unwrap();
    Folders::write(root.path(), "@AMML/Loader", &Folders::manifest("AMML", "Loader"));
    Folders::write(root.path(), "@amml/fine", &Folders::manifest("amml", "fine"));

    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(root.path()).unwrap();

    assert_eq!(scan.mods.len(), 1);
    assert_eq!(scan.mods[0].name, "fine");
    assert!(scan.skipped[0].dir.ends_with("Loader"));
}

#[test]
fn a_flat_folder_with_a_manifest_is_skipped_and_told_where_to_move() {
    let root = tempdir().unwrap();
    Folders::write(root.path(), "zoom", &Folders::manifest("acme", "zoom"));
    Folders::write(root.path(), "@ns/fine", &Folders::manifest("ns", "fine"));

    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(root.path()).unwrap();

    assert_eq!(scan.mods.len(), 1);
    assert_eq!(scan.skipped.len(), 1);
    assert!(scan.skipped[0].reason.contains("@acme/zoom"), "{}", scan.skipped[0].reason);
}

#[test]
fn a_folder_that_does_not_match_its_manifest_is_skipped() {
    let root = tempdir().unwrap();
    Folders::write(root.path(), "@acme/zoom", &Folders::manifest("other", "zoom"));

    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(root.path()).unwrap();

    assert!(scan.mods.is_empty());
    let reason = &scan.skipped[0].reason;
    assert!(reason.contains("@acme/zoom") && reason.contains("@other/zoom"), "{reason}");
}

#[test]
fn the_folder_match_ignores_case_and_a_second_copy_is_a_duplicate() {
    let root = tempdir().unwrap();
    Folders::write(root.path(), "@Acme/Zoom", &Folders::manifest("acme", "zoom"));
    // A case-insensitive file system holds only one of the two folders.
    let second_copy_exists = !root.path().join("@acme").join("zoom").exists();
    if second_copy_exists {
        Folders::write(root.path(), "@acme/zoom", &Folders::manifest("acme", "zoom"));
    }

    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(root.path()).unwrap();

    assert_eq!(scan.mods.len(), 1);
    assert!(scan.mods[0].dir.ends_with("Zoom"));
    if second_copy_exists {
        assert!(scan.skipped[0].reason.contains("another folder already holds"), "{}", scan.skipped[0].reason);
    } else {
        assert!(scan.skipped.is_empty());
    }
}

#[test]
fn the_same_name_in_two_namespaces_loads_twice() {
    let root = tempdir().unwrap();
    Folders::write(root.path(), "@a/map", &Folders::manifest("a", "map"));
    Folders::write(root.path(), "@b/map", &Folders::manifest("b", "map"));

    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(root.path()).unwrap();

    assert_eq!(scan.mods.len(), 2);
    assert!(scan.skipped.is_empty());
}

#[test]
fn a_missing_namespace_is_refused() {
    let root = tempdir().unwrap();
    Folders::write(root.path(), "@acme/zoom", "name = \"zoom\"\nentry = \"zoom.dll\"\napi_version = 2\n");

    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(root.path()).unwrap();

    assert!(scan.mods.is_empty());
    assert!(scan.skipped[0].reason.contains("`namespace` is missing"), "{}", scan.skipped[0].reason);
}

#[test]
fn a_namespace_folder_without_mods_is_ignored() {
    let root = tempdir().unwrap();
    fs::create_dir_all(root.path().join("@empty")).unwrap();

    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(root.path()).unwrap();

    assert!(scan.mods.is_empty() && scan.skipped.is_empty());
}
