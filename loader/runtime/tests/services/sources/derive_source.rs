use std::sync::atomic::Ordering;

use amml_data::{BuildData, BuildDataStore, EntryKind, Fingerprint, RecipeLoader};
use amml_runtime::{CacheFile, SourceNote};

use crate::support::Fixtures;

#[test]
fn derive_source_writes_the_cache_file_for_the_build() {
    let dir = tempfile::tempdir().unwrap();
    let probe = Fixtures::probe_dir(dir.path());
    let (source, _) = Fixtures::probe_derive(&probe);
    let outcome = source.resolve_all(&[Fixtures::id("Probe.run"), Fixtures::id("Probe.absent")]);

    assert_eq!(outcome.found.len(), 1);
    let found = outcome.found[&Fixtures::id("Probe.run")];
    assert_eq!(found.kind, EntryKind::Function);
    assert_eq!(found.value, u64::from(probe.rva));
    assert_eq!(outcome.settled.iter().cloned().collect::<Vec<_>>(), vec![Fixtures::id("Probe.absent")]);
    assert!(matches!(outcome.notes.as_slice(), [SourceNote::DeriveRan { hits: 0, derived: 2, .. }]));

    let build = Fixtures::probe_build();
    let written = BuildDataStore::read(&CacheFile::path(&probe.cache, build)).unwrap();
    assert_eq!(written.build, build);
    assert_eq!(written.entries.get(&Fixtures::id("Probe.run")), Some(&found));
    assert!(written.failed.contains_key(&Fixtures::id("Probe.absent")));
    let prints = Fixtures::prints(&RecipeLoader::load_dir(&probe.recipes).unwrap());
    assert_eq!(written.fingerprints.get(&Fixtures::id("Probe.run")), prints.get(&Fixtures::id("Probe.run")));
}

#[test]
fn derive_runs_only_the_misses_and_merges_the_cache() {
    let dir = tempfile::tempdir().unwrap();
    let probe = Fixtures::probe_dir(dir.path());
    let set = RecipeLoader::load_dir(&probe.recipes).unwrap();
    let prints = Fixtures::prints(&set);
    let build = Fixtures::probe_build();
    // A = Probe.global with a value a derive would not give; X is no longer in the set; B = Probe.run is stale
    let mut old = BuildData::new(build);
    old.entries.insert(Fixtures::id("Probe.global"), Fixtures::resolved(EntryKind::Global, 0x1234));
    old.fingerprints.insert(Fixtures::id("Probe.global"), prints[&Fixtures::id("Probe.global")]);
    old.entries.insert(Fixtures::id("Gone.entry"), Fixtures::resolved(EntryKind::Global, 0x55));
    old.fingerprints.insert(Fixtures::id("Gone.entry"), Fingerprint(1));
    old.entries.insert(Fixtures::id("Probe.run"), Fixtures::resolved(EntryKind::Function, 0x77));
    old.fingerprints.insert(Fixtures::id("Probe.run"), Fingerprint(prints[&Fixtures::id("Probe.run")].0 ^ 1));
    std::fs::create_dir_all(&probe.cache).unwrap();
    BuildDataStore::write(&CacheFile::path(&probe.cache, build), &old).unwrap();

    let (source, _) = Fixtures::probe_derive(&probe);
    let outcome = source.resolve_all(&[Fixtures::id("Probe.run"), Fixtures::id("Probe.absent")]);
    assert_eq!(
        outcome.found.get(&Fixtures::id("Probe.run")).map(|resolved| resolved.value),
        Some(u64::from(probe.rva))
    );
    assert!(matches!(outcome.notes.as_slice(), [SourceNote::DeriveRan { hits: 0, derived: 2, .. }]));

    let written = BuildDataStore::read(&CacheFile::path(&probe.cache, build)).unwrap();
    assert_eq!(
        written.entries.keys().cloned().collect::<Vec<_>>(),
        vec![Fixtures::id("Probe.global"), Fixtures::id("Probe.run")]
    );
    assert_eq!(written.entries[&Fixtures::id("Probe.global")].value, 0x1234);
    assert_eq!(written.entries[&Fixtures::id("Probe.run")].value, u64::from(probe.rva));
    assert_eq!(written.failed.keys().cloned().collect::<Vec<_>>(), vec![Fixtures::id("Probe.absent")]);
    assert_eq!(written.fingerprints, prints);
}

#[test]
fn shipped_hits_are_not_copied_into_the_cache() {
    let dir = tempfile::tempdir().unwrap();
    let probe = Fixtures::probe_dir(dir.path());
    let set = RecipeLoader::load_dir(&probe.recipes).unwrap();
    let build = Fixtures::probe_build();
    Fixtures::write_data(
        &CacheFile::path(&probe.shipped, build),
        build,
        &set,
        vec![("Probe.global", Fixtures::resolved(EntryKind::Global, 0x1234))],
        Vec::new(),
    );

    let (source, _) = Fixtures::probe_derive(&probe);
    let outcome = source.resolve_all(&[Fixtures::id("Probe.global"), Fixtures::id("Probe.run")]);
    assert_eq!(outcome.found.get(&Fixtures::id("Probe.global")).map(|resolved| resolved.value), Some(0x1234));
    assert!(matches!(outcome.notes.as_slice(), [SourceNote::DeriveRan { hits: 1, derived: 1, .. }]));

    let written = BuildDataStore::read(&CacheFile::path(&probe.cache, build)).unwrap();
    assert_eq!(written.entries.keys().cloned().collect::<Vec<_>>(), vec![Fixtures::id("Probe.run")]);
}

#[test]
fn the_cache_file_is_replaced_atomically() {
    let dir = tempfile::tempdir().unwrap();
    let probe = Fixtures::probe_dir(dir.path());
    let set = RecipeLoader::load_dir(&probe.recipes).unwrap();
    let build = Fixtures::probe_build();
    Fixtures::write_cache(
        &probe.cache,
        build,
        build,
        &set,
        vec![("Probe.global", Fixtures::resolved(EntryKind::Global, 9))],
    );

    let (source, _) = Fixtures::probe_derive(&probe);
    source.resolve_all(&[Fixtures::id("Probe.run")]);
    let names: Vec<String> = std::fs::read_dir(&probe.cache)
        .unwrap()
        .map(|item| item.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, vec![format!("{}.hcl", build.file_stem())]);
    let written = BuildDataStore::read(&CacheFile::path(&probe.cache, build)).unwrap();
    assert_eq!(written.entries.len(), 2);
}

#[test]
fn run_full_derives_every_entry_and_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let probe = Fixtures::probe_dir(dir.path());
    let set = RecipeLoader::load_dir(&probe.recipes).unwrap();
    let build = Fixtures::probe_build();
    Fixtures::write_data(
        &CacheFile::path(&probe.shipped, build),
        build,
        &set,
        vec![("Probe.global", Fixtures::resolved(EntryKind::Global, 0x1234))],
        Vec::new(),
    );

    let (source, runs) = Fixtures::probe_derive(&probe);
    let outcome = source.run_full();
    assert_eq!(runs.load(Ordering::Relaxed), 1);
    assert_eq!(
        outcome.found.get(&Fixtures::id("Probe.global")).map(|resolved| resolved.value),
        Some(u64::from(probe.rva))
    );
    assert!(outcome.found.contains_key(&Fixtures::id("Probe.run")));
    assert!(!probe.cache.exists());
}
