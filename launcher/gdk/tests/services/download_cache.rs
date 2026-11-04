use amml_gdk::DownloadCache;

#[test]
fn retain_only_keeps_named_and_deletes_rest() {
    let dir = tempfile::tempdir().unwrap();
    for name in ["a.msixvc", "b.msixvc", "b.msixvc.part", "c.msixvc.part.meta"] {
        std::fs::write(dir.path().join(name), b"x").unwrap();
    }
    let cache = DownloadCache::new(dir.path().to_path_buf());

    let mut deleted: Vec<String> = cache
        .retain_only(&["a.msixvc"])
        .unwrap()
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
        .collect();
    deleted.sort();

    assert_eq!(deleted, vec!["b.msixvc", "b.msixvc.part", "c.msixvc.part.meta"]);
    let left: Vec<_> = std::fs::read_dir(dir.path()).unwrap().map(|e| e.unwrap().file_name()).collect();
    assert_eq!(left, vec![std::ffi::OsString::from("a.msixvc")]);
}

#[test]
fn cached_requires_exact_size() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.msixvc"), [0u8; 10]).unwrap();
    let cache = DownloadCache::new(dir.path().to_path_buf());
    assert!(cache.cached("a.msixvc", 10).is_some());
    assert!(cache.cached("a.msixvc", 11).is_none());
}
