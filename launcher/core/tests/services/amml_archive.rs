use crate::support::Zips;
use amml_launcher_core::{AmmlArchive, LauncherError};

#[test]
fn a_release_zip_opens_and_reads_its_version() {
    let dir = tempfile::tempdir().unwrap();
    let zip = Zips::amml_zip(&dir.path().join("amml.zip"), "26.60.1\r\n", true);
    let archive = AmmlArchive::open(&zip).unwrap();
    assert_eq!(archive.version(), "26.60.1");

    let dest = dir.path().join("out");
    archive.extract_to(&dest).unwrap();
    assert!(dest.join("amml.dll").is_file());
    assert!(dest.join("amml-patch.exe").is_file());
    assert!(dest.join("amml").join("recipes").join("sub").join("b.hcl").is_file());
}

#[test]
fn a_zip_without_the_patcher_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let zip = Zips::amml_zip(&dir.path().join("amml.zip"), "26.60.1", false);
    match AmmlArchive::open(&zip) {
        Err(LauncherError::BadArchive { reason, .. }) => assert!(reason.contains("amml-patch.exe"), "{reason}"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_zip_without_recipes_or_a_version_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let no_recipes = Zips::raw(
        &dir.path().join("a.zip"),
        &[("amml.dll", b"x"), ("amml-patch.exe", b"x"), ("amml/amml-version.txt", b"1.0.0")],
    );
    assert!(matches!(AmmlArchive::open(&no_recipes), Err(LauncherError::BadArchive { .. })));
    let no_version = Zips::raw(
        &dir.path().join("b.zip"),
        &[("amml.dll", b"x"), ("amml-patch.exe", b"x"), ("amml/recipes/a.hcl", b"x")],
    );
    assert!(matches!(AmmlArchive::open(&no_version), Err(LauncherError::BadArchive { .. })));
}

#[test]
fn a_zip_that_writes_outside_its_folder_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let zip = Zips::raw(
        &dir.path().join("a.zip"),
        &[
            ("amml.dll", b"x"),
            ("amml-patch.exe", b"x"),
            ("amml/recipes/a.hcl", b"x"),
            ("amml/amml-version.txt", b"1.0.0"),
            ("../evil.dll", b"x"),
        ],
    );
    match AmmlArchive::open(&zip) {
        Err(LauncherError::BadArchive { file, .. }) => assert_eq!(file.to_string_lossy(), "../evil.dll"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn an_archive_without_data_is_accepted() {
    let dir = tempfile::tempdir().unwrap();
    let zip = Zips::amml_zip(&dir.path().join("amml.zip"), "26.60.1", true);
    let archive = AmmlArchive::open(&zip).unwrap();
    let dest = dir.path().join("out");
    archive.extract_to(&dest).unwrap();
    assert!(!dest.join("amml").join("data").exists());
}

#[test]
fn an_archive_with_data_extracts_it() {
    let dir = tempfile::tempdir().unwrap();
    let zip = Zips::raw(
        &dir.path().join("a.zip"),
        &[
            ("amml.dll", b"x"),
            ("amml-patch.exe", b"x"),
            ("amml/recipes/a.hcl", b"x"),
            ("amml/data/build-one.hcl", b"solved"),
            ("amml/amml-version.txt", b"1.0.0"),
        ],
    );
    let archive = AmmlArchive::open(&zip).unwrap();
    let dest = dir.path().join("out");
    archive.extract_to(&dest).unwrap();
    assert_eq!(std::fs::read(dest.join("amml").join("data").join("build-one.hcl")).unwrap(), b"solved");
}
