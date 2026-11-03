use crate::support::Zips;
use amml_launcher_core::{LauncherError, ModArchive};
use std::path::Path;

fn all_files(dir: &Path) -> Vec<String> {
    let mut found = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(next) = stack.pop() {
        for entry in std::fs::read_dir(&next).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                stack.push(entry.path());
            } else {
                found.push(entry.path().strip_prefix(dir).unwrap().to_string_lossy().replace('\\', "/"));
            }
        }
    }
    found.sort();
    found
}

#[test]
fn a_zip_with_one_mod_opens_and_extracts_its_folder() {
    let dir = tempfile::tempdir().unwrap();
    let zip = Zips::mod_zip(
        &dir.path().join("zoom.zip"),
        "alaydriem",
        "zoom",
        "26.60.1",
        2,
        &[("config/keys.hcl", b"keys"), ("assets/a/b.png", b"png")],
    );
    let archive = ModArchive::open(&zip).unwrap();
    assert_eq!(archive.id.as_str(), "@alaydriem/zoom");
    assert_eq!(archive.api_version(), 2);
    assert_eq!(archive.manifest.version, "26.60.1");

    let dest = dir.path().join("out");
    archive.extract_to(&dest).unwrap();
    assert_eq!(all_files(&dest), vec!["amml.hcl", "assets/a/b.png", "config/keys.hcl", "zoom.dll"]);
    assert_eq!(std::fs::read(dest.join("config").join("keys.hcl")).unwrap(), b"keys");
}

#[test]
fn two_mods_or_none_are_refused() {
    let dir = tempfile::tempdir().unwrap();
    let manifest = |name: &str| Zips::manifest("a", name, Some("1.0.0"), 2);
    let (one, two) = (manifest("one"), manifest("two"));
    let two_mods = Zips::raw(
        &dir.path().join("two.zip"),
        &[("@a/one/amml.hcl", one.as_bytes()), ("@a/two/amml.hcl", two.as_bytes())],
    );
    assert!(matches!(ModArchive::open(&two_mods), Err(LauncherError::BadArchive { .. })));

    let two_namespaces = Zips::raw(
        &dir.path().join("ns.zip"),
        &[("@a/one/amml.hcl", one.as_bytes()), ("@b/one/amml.hcl", one.as_bytes())],
    );
    assert!(matches!(ModArchive::open(&two_namespaces), Err(LauncherError::BadArchive { .. })));

    let none = Zips::raw(&dir.path().join("none.zip"), &[]);
    assert!(matches!(ModArchive::open(&none), Err(LauncherError::BadArchive { .. })));

    let loose = Zips::raw(&dir.path().join("loose.zip"), &[("readme.txt", b"x")]);
    assert!(matches!(ModArchive::open(&loose), Err(LauncherError::BadArchive { .. })));

    let no_manifest = Zips::raw(&dir.path().join("nm.zip"), &[("@a/one/one.dll", b"x")]);
    assert!(matches!(ModArchive::open(&no_manifest), Err(LauncherError::BadArchive { .. })));
}

#[test]
fn a_manifest_naming_another_mod_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let manifest = Zips::manifest("a", "c", Some("1.0.0"), 2);
    let zip = Zips::raw(&dir.path().join("x.zip"), &[("@a/b/amml.hcl", manifest.as_bytes())]);
    match ModArchive::open(&zip) {
        Err(LauncherError::BadArchive { reason, .. }) => assert!(reason.contains("@a/c"), "{reason}"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_folder_name_that_is_not_a_mod_id_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let manifest = Zips::manifest("A", "b", Some("1.0.0"), 2);
    let zip = Zips::raw(&dir.path().join("x.zip"), &[("@A/b/amml.hcl", manifest.as_bytes())]);
    assert!(matches!(ModArchive::open(&zip), Err(LauncherError::BadModId(_))));
}

#[test]
fn entries_that_escape_are_refused() {
    let dir = tempfile::tempdir().unwrap();
    let manifest = Zips::manifest("a", "b", Some("1.0.0"), 2);
    for bad in ["@a/b/../../x.dll", "/x", "C:/x", "@a\\b\\x", "@a/b/./x", "@a/b//x"] {
        let zip = Zips::raw(&dir.path().join("bad.zip"), &[("@a/b/amml.hcl", manifest.as_bytes()), (bad, b"evil")]);
        let outcome = ModArchive::open(&zip);
        match outcome {
            Err(LauncherError::BadArchive { file, .. }) => assert_eq!(file.to_string_lossy(), bad, "{bad}"),
            other => panic!("{bad}: {other:?}"),
        }
    }
    assert!(!dir.path().join("x.dll").exists());
    assert!(!dir.path().parent().unwrap().join("x.dll").exists());
}

#[test]
fn the_listing_is_read_from_the_zip() {
    let dir = tempfile::tempdir().unwrap();
    let zip = Zips::mod_zip(&dir.path().join("z.zip"), "alaydriem", "zoom", "26.60.1", 2, &[]);
    let listing = ModArchive::open(&zip).unwrap().manifest.listing.unwrap();
    assert_eq!(listing.title, "ZOOM");
    assert_eq!(listing.summary, "ZOOM summary");
}

#[test]
fn an_archive_with_too_many_entries_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let manifest = Zips::manifest("a", "b", Some("1.0.0"), 2);
    let mut entries: Vec<(String, Vec<u8>)> = vec![("@a/b/amml.hcl".to_string(), manifest.into_bytes())];
    for n in 0..20_001 {
        entries.push((format!("@a/b/f{n}"), Vec::new()));
    }
    let borrowed: Vec<(&str, &[u8])> = entries.iter().map(|(n, b)| (n.as_str(), b.as_slice())).collect();
    let zip = Zips::raw(&dir.path().join("many.zip"), &borrowed);
    match ModArchive::open(&zip) {
        Err(LauncherError::BadArchive { reason, .. }) => assert!(reason.contains("20000"), "{reason}"),
        other => panic!("{other:?}"),
    }
}
