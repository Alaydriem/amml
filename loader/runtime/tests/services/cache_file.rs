use amml_data::BuildId;
use amml_runtime::CacheFile;

#[test]
fn the_cache_file_is_named_by_the_build_only() {
    let dir = std::path::Path::new("cache");
    assert_eq!(
        CacheFile::path(dir, BuildId::new(0x6aed1906, 0xb127418e74d2f2f9)),
        dir.join("6aed1906-b127418e74d2f2f9.hcl")
    );
}

#[test]
fn remove_stale_deletes_only_old_digest_named_files() {
    let dir = tempfile::tempdir().unwrap();
    let keep = [
        "6aed1906-b127418e74d2f2f9.hcl",
        "6aed1906-b127418e74d2f2f9-0123.hcl",
        "6aed1906-b127418e74d2f2f9-0123456789abcdef.txt",
        "6aed1906-b127418e74d2f2f9-0123456789abcdeg.hcl",
        "notes.hcl",
    ];
    let stale = ["6aed1906-b127418e74d2f2f9-0123456789abcdef.hcl", "6ad79ee4-17d51891a1119914-fedcba9876543210.hcl"];
    for name in keep.iter().chain(stale.iter()) {
        std::fs::write(dir.path().join(name), "x").unwrap();
    }
    // a folder with a stale-looking name is not a file
    std::fs::create_dir(dir.path().join("6aea437e-7eff13daabb430da-0000000000000000.hcl")).unwrap();

    assert_eq!(CacheFile::remove_stale(dir.path()), 2);
    let mut left: Vec<String> = std::fs::read_dir(dir.path())
        .unwrap()
        .map(|item| item.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    left.sort();
    let mut expected: Vec<String> = keep.iter().map(|name| name.to_string()).collect();
    expected.push("6aea437e-7eff13daabb430da-0000000000000000.hcl".to_string());
    expected.sort();
    assert_eq!(left, expected);
}

#[test]
fn remove_stale_on_a_missing_folder_removes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(CacheFile::remove_stale(&dir.path().join("none")), 0);
}
