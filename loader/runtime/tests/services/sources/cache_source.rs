use amml_data::{BuildData, BuildDataStore, BuildId, EntryKind, Fingerprint, RecipeSet};
use amml_runtime::{CacheFile, CacheSource, SourceNote};

use crate::support::Fixtures;

const BUILD: BuildId = BuildId { timestamp: 2, text_hash: 7 };

fn set() -> RecipeSet {
    Fixtures::set(vec![
        Fixtures::entry("A", EntryKind::Function, Some("A")),
        Fixtures::entry("B", EntryKind::Global, Some("B")),
        Fixtures::entry("C", EntryKind::Field, None),
    ])
}

struct Dirs {
    _root: tempfile::TempDir,
    cache: std::path::PathBuf,
    shipped: std::path::PathBuf,
}

fn dirs() -> Dirs {
    let root = tempfile::tempdir().unwrap();
    let cache = root.path().join("cache");
    let shipped = root.path().join("data");
    Dirs { _root: root, cache, shipped }
}

fn source(dirs: &Dirs) -> CacheSource {
    CacheSource::new(dirs.cache.clone(), dirs.shipped.clone(), BUILD, set())
}

fn wanted() -> Vec<amml_data::EntryId> {
    vec![Fixtures::id("A"), Fixtures::id("B"), Fixtures::id("C")]
}

#[test]
fn hits_come_from_the_cache_or_the_shipped_file() {
    let dirs = dirs();
    Fixtures::write_cache(
        &dirs.cache,
        BUILD,
        BUILD,
        &set(),
        vec![("A", Fixtures::resolved(EntryKind::Function, 0x100))],
    );
    Fixtures::write_data(
        &CacheFile::path(&dirs.shipped, BUILD),
        BUILD,
        &set(),
        vec![("B", Fixtures::resolved(EntryKind::Global, 0x200))],
        Vec::new(),
    );

    let outcome = source(&dirs).resolve_all(&wanted());
    assert_eq!(outcome.found.get(&Fixtures::id("A")).map(|resolved| resolved.value), Some(0x100));
    assert_eq!(outcome.found.get(&Fixtures::id("B")).map(|resolved| resolved.value), Some(0x200));
    assert!(!outcome.found.contains_key(&Fixtures::id("C")));
    assert!(outcome.notes.is_empty());
}

#[test]
fn a_known_failure_is_settled_and_not_found() {
    let dirs = dirs();
    Fixtures::write_data(&CacheFile::path(&dirs.cache, BUILD), BUILD, &set(), Vec::new(), vec!["C"]);

    let outcome = source(&dirs).resolve_all(&wanted());
    assert!(outcome.found.is_empty());
    assert_eq!(outcome.settled.iter().cloned().collect::<Vec<_>>(), vec![Fixtures::id("C")]);
}

#[test]
fn a_hit_needs_an_equal_fingerprint() {
    let dirs = dirs();
    let mut data = BuildData::new(BUILD);
    data.entries.insert(Fixtures::id("A"), Fixtures::resolved(EntryKind::Function, 0x100));
    data.fingerprints.insert(Fixtures::id("A"), Fingerprint(0x1234));
    // B has no fingerprint at all
    data.entries.insert(Fixtures::id("B"), Fixtures::resolved(EntryKind::Global, 0x200));
    std::fs::create_dir_all(&dirs.cache).unwrap();
    BuildDataStore::write(&CacheFile::path(&dirs.cache, BUILD), &data).unwrap();

    let outcome = source(&dirs).resolve_all(&wanted());
    assert!(outcome.found.is_empty());
    assert!(outcome.settled.is_empty());
}

#[test]
fn a_format_1_or_foreign_or_unreadable_file_gives_no_hits() {
    // format 1: values without fingerprints
    let format_1 = dirs();
    std::fs::create_dir_all(&format_1.cache).unwrap();
    std::fs::write(
        CacheFile::path(&format_1.cache, BUILD),
        "format = 1\n\nbuild {\n  timestamp = \"0x00000002\"\n  text_hash = \"0x0000000000000007\"\n}\n\n\
         resolved \"A\" {\n  kind    = \"function\"\n  value   = \"0x100\"\n  recipes = 2\n}\n",
    )
    .unwrap();
    let outcome = source(&format_1).resolve_all(&wanted());
    assert!(outcome.found.is_empty());
    assert!(outcome.notes.is_empty());

    // foreign: the file of this build holds another build
    let foreign = dirs();
    let other = BuildId::new(1, 7);
    Fixtures::write_cache(
        &foreign.cache,
        BUILD,
        other,
        &set(),
        vec![("A", Fixtures::resolved(EntryKind::Function, 1))],
    );
    Fixtures::write_data(
        &CacheFile::path(&foreign.shipped, BUILD),
        other,
        &set(),
        vec![("B", Fixtures::resolved(EntryKind::Global, 2))],
        Vec::new(),
    );
    let outcome = source(&foreign).resolve_all(&wanted());
    assert!(outcome.found.is_empty());
    assert_eq!(
        outcome.notes,
        vec![
            SourceNote::CacheForeignBuild { found: other.file_stem(), expected: BUILD.file_stem() },
            SourceNote::ShippedForeignBuild { found: other.file_stem(), expected: BUILD.file_stem() },
        ]
    );

    // unreadable: a truncated cache file
    let truncated = dirs();
    Fixtures::write_cache(
        &truncated.cache,
        BUILD,
        BUILD,
        &set(),
        vec![("A", Fixtures::resolved(EntryKind::Function, 1))],
    );
    let path = CacheFile::path(&truncated.cache, BUILD);
    let text = std::fs::read_to_string(&path).unwrap();
    std::fs::write(&path, &text[..text.len() - 12]).unwrap();
    let outcome = source(&truncated).resolve_all(&wanted());
    assert!(outcome.found.is_empty());
    assert!(matches!(outcome.notes.as_slice(), [SourceNote::CacheUnreadable(_)]));
}

#[test]
fn a_missing_cache_file_is_noted_and_a_missing_shipped_file_is_not() {
    let dirs = dirs();
    let outcome = source(&dirs).resolve_all(&wanted());
    assert!(outcome.found.is_empty());
    assert_eq!(outcome.notes, vec![SourceNote::CacheMissing]);
}

#[test]
fn only_the_running_builds_shipped_file_is_read() {
    let dirs = dirs();
    let other = BuildId::new(1, 9);
    Fixtures::write_data(
        &CacheFile::path(&dirs.shipped, other),
        other,
        &set(),
        vec![("A", Fixtures::resolved(EntryKind::Function, 0x100))],
        Vec::new(),
    );
    Fixtures::write_data(
        &CacheFile::path(&dirs.shipped, BUILD),
        BUILD,
        &set(),
        vec![("B", Fixtures::resolved(EntryKind::Global, 0x200))],
        Vec::new(),
    );

    let outcome = source(&dirs).resolve_all(&wanted());
    assert_eq!(outcome.found.keys().cloned().collect::<Vec<_>>(), vec![Fixtures::id("B")]);
}

#[test]
fn entries_with_the_wrong_kind_or_outside_the_set_are_dropped() {
    let dirs = dirs();
    Fixtures::write_cache(
        &dirs.cache,
        BUILD,
        BUILD,
        &set(),
        vec![
            ("A", Fixtures::resolved(EntryKind::Global, 0x100)),
            ("B", Fixtures::resolved(EntryKind::Global, 0x200)),
            ("Unknown", Fixtures::resolved(EntryKind::Function, 0x300)),
        ],
    );
    let outcome = source(&dirs).resolve_all(&[Fixtures::id("A"), Fixtures::id("B"), Fixtures::id("Unknown")]);
    assert_eq!(outcome.found.keys().cloned().collect::<Vec<_>>(), vec![Fixtures::id("B")]);
    assert_eq!(outcome.notes, vec![SourceNote::CacheKindMismatch(Fixtures::id("A"))]);
}
