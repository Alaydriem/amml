use amml_launcher_core::{CacheLock, LauncherError, RegistryCache, RegistryRef};
use std::path::Path;

fn registry() -> RegistryRef {
    RegistryRef::parse("https://example.org/reg/").unwrap()
}

fn file_count(dir: &Path) -> usize {
    let mut count = 0;
    for entry in std::fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            count += file_count(&entry.path());
        } else {
            count += 1;
        }
    }
    count
}

#[test]
fn files_round_trip_with_their_etag() {
    let dir = tempfile::tempdir().unwrap();
    let cache = RegistryCache::new(dir.path().to_path_buf(), RegistryCache::BUDGET, CacheLock::new());
    cache.write(&registry(), "mods/@a/b.json", b"{\"x\":1}", Some("\"e1\"")).unwrap();
    let (bytes, etag) = cache.read(&registry(), "mods/@a/b.json").unwrap();
    assert_eq!(bytes, b"{\"x\":1}");
    assert_eq!(etag.as_deref(), Some("\"e1\""));

    cache.write(&registry(), "mods/@a/b.json", b"{}", None).unwrap();
    let (_, etag) = cache.read(&registry(), "mods/@a/b.json").unwrap();
    assert_eq!(etag, None);
    assert!(cache.read(&registry(), "mods/@a/other.json").is_none());
    // the write goes through a temporary file that is gone again
    assert_eq!(file_count(dir.path()), 1);
}

#[test]
fn the_budget_evicts_the_least_recently_read_file() {
    let dir = tempfile::tempdir().unwrap();
    let cache = RegistryCache::new(dir.path().to_path_buf(), 1_000, CacheLock::new());
    let reg = registry();
    cache.write(&reg, "a", &[1u8; 330], None).unwrap();
    cache.write(&reg, "b", &[2u8; 330], None).unwrap();
    cache.write(&reg, "c", &[3u8; 330], None).unwrap();
    assert!(cache.read(&reg, "a").is_some());
    cache.write(&reg, "d", &[4u8; 400], None).unwrap();

    assert!(cache.read(&reg, "b").is_none());
    assert!(cache.read(&reg, "c").is_none());
    assert!(cache.read(&reg, "a").is_some());
    assert!(cache.read(&reg, "d").is_some());
}

#[test]
fn the_budget_counts_every_registry_and_never_drops_the_new_file() {
    let dir = tempfile::tempdir().unwrap();
    let cache = RegistryCache::new(dir.path().to_path_buf(), 500, CacheLock::new());
    let other = RegistryRef::parse("https://other.org/").unwrap();
    cache.write(&other, "old", &[1u8; 300], None).unwrap();
    cache.write(&registry(), "huge", &[2u8; 900], None).unwrap();
    assert!(cache.read(&other, "old").is_none());
    assert!(cache.read(&registry(), "huge").is_some());
}

#[test]
fn hostile_relative_paths_are_refused() {
    let dir = tempfile::tempdir().unwrap();
    let inner = dir.path().join("cache");
    std::fs::create_dir_all(&inner).unwrap();
    let cache = RegistryCache::new(inner.clone(), RegistryCache::BUDGET, CacheLock::new());
    for bad in [
        "../x",
        "mods/@a/../../x",
        "/abs",
        "C:/x",
        "mods//x",
        "",
        "a\\b",
        "./x",
        "icons/CON",
        "icons/nul.png",
        "icons/COM1",
        "icons/a.",
        "icons/a.png.etag",
        "icons/a.png.tmp",
    ] {
        assert!(matches!(RegistryCache::safe_relative(bad), Err(LauncherError::BadModId(_))), "{bad}");
        assert!(matches!(cache.write(&registry(), bad, b"x", None), Err(LauncherError::BadModId(_))), "{bad}");
        assert!(cache.read(&registry(), bad).is_none(), "{bad}");
    }
    assert_eq!(file_count(dir.path()), 0);
    assert_eq!(RegistryCache::safe_relative("mods/@a/b.json").unwrap(), "mods/@a/b.json");
}

#[test]
fn two_caches_on_one_folder_never_pair_a_body_with_another_writes_etag() {
    let dir = tempfile::tempdir().unwrap();
    let lock = CacheLock::new();
    let make = || RegistryCache::new(dir.path().to_path_buf(), RegistryCache::BUDGET, lock.clone());
    let (cache_a, cache_b, reader) = (make(), make(), make());
    let reg = registry();
    cache_a.write(&reg, "root.json", b"A", Some("etag-a")).unwrap();
    std::thread::scope(|scope| {
        scope.spawn(|| {
            for _ in 0..500 {
                cache_a.write(&reg, "root.json", b"A", Some("etag-a")).unwrap();
            }
        });
        scope.spawn(|| {
            for _ in 0..500 {
                cache_b.write(&reg, "root.json", b"B", Some("etag-b")).unwrap();
            }
        });
        scope.spawn(|| {
            for _ in 0..500 {
                let (bytes, etag) = reader.read(&reg, "root.json").unwrap();
                let pair = (bytes.as_slice(), etag.as_deref());
                assert!(
                    pair == (b"A".as_slice(), Some("etag-a")) || pair == (b"B".as_slice(), Some("etag-b")),
                    "mixed pair: {pair:?}"
                );
            }
        });
    });
}
