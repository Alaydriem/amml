use amml_data::{BuildDataStore, EntryKind};
use amml_runtime::{CacheFile, DeriveSource, RecipeDigest};

use crate::support::Fixtures;

#[test]
fn derive_source_writes_the_cache_file_for_the_build_and_recipes() {
    let dir = tempfile::tempdir().unwrap();
    let probe = Fixtures::probe_dir(dir.path());
    let source = DeriveSource::new(Fixtures::probe_dll(), probe.recipes.clone(), probe.data.clone());
    let outcome = source.resolve_all(&[Fixtures::id("Probe.run"), Fixtures::id("Probe.absent")]);

    assert_eq!(outcome.found.len(), 1);
    let found = outcome.found[&Fixtures::id("Probe.run")];
    assert_eq!(found.kind, EntryKind::Function);
    assert_eq!(found.value, u64::from(probe.rva));
    assert!(outcome.notes.is_empty());

    let build = Fixtures::probe_build();
    let digest = RecipeDigest::of_dir(&probe.recipes).unwrap();
    let written = BuildDataStore::read(&CacheFile::path(&probe.data, build, &digest)).unwrap();
    assert_eq!(written.build, build);
    assert_eq!(written.entries.get(&Fixtures::id("Probe.run")), Some(&found));
    assert!(!written.entries.contains_key(&Fixtures::id("Probe.absent")));
}
