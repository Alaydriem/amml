use std::sync::atomic::Ordering;

use amml_data::{BuildId, EntryKind};
use amml_runtime::{CacheFile, CacheSource, DeriveSource, LeviLaminaSource, ResolverChain, ResolverSource, SourceNote};

use crate::support::{BASE, Fixtures};

fn lookup(name: &str) -> Option<u64> {
    match name {
        "A" => Some(BASE + 0x999),
        "B" => Some(BASE + 0x200),
        _ => None,
    }
}

fn set() -> amml_data::RecipeSet {
    Fixtures::set(vec![
        Fixtures::entry("A", EntryKind::Function, Some("A")),
        Fixtures::entry("B", EntryKind::Function, Some("B")),
        Fixtures::entry("C", EntryKind::Function, Some("C")),
        Fixtures::entry("F", EntryKind::Field, None),
    ])
}

fn cache(dir: &std::path::Path, build: BuildId) -> ResolverSource {
    ResolverSource::Cache(CacheSource::new(dir.to_path_buf(), dir.join("shipped"), build, set()))
}

fn levilamina() -> ResolverSource {
    ResolverSource::LeviLamina(LeviLaminaSource::new(lookup, set(), BASE, Fixtures::layout()))
}

#[test]
fn cache_for_another_build_is_ignored_and_noted() {
    let dir = tempfile::tempdir().unwrap();
    let (old, current) = (BuildId::new(1, 7), BuildId::new(2, 7));
    Fixtures::write_cache(
        dir.path(),
        current,
        old,
        &set(),
        vec![("A", Fixtures::resolved(EntryKind::Function, 0x100))],
    );
    let table = ResolverChain::new(vec![cache(dir.path(), current)]).resolve(&[Fixtures::id("A")]);
    assert_eq!(table.missing(), vec![Fixtures::id("A")]);
    assert_eq!(table.get(&Fixtures::id("A")), None);
    assert_eq!(
        table.notes(),
        [SourceNote::CacheForeignBuild { found: old.file_stem(), expected: current.file_stem() }]
    );
}

#[test]
fn a_missing_cache_file_is_noted() {
    let dir = tempfile::tempdir().unwrap();
    let table = ResolverChain::new(vec![cache(dir.path(), BuildId::new(2, 7))]).resolve(&[Fixtures::id("A")]);
    assert_eq!(table.notes(), [SourceNote::CacheMissing]);
}

#[test]
fn later_sources_fill_only_what_earlier_ones_missed() {
    let dir = tempfile::tempdir().unwrap();
    let build = BuildId::new(2, 7);
    Fixtures::write_cache(
        dir.path(),
        build,
        build,
        &set(),
        vec![("A", Fixtures::resolved(EntryKind::Function, 0x100))],
    );
    let chain = ResolverChain::new(vec![cache(dir.path(), build), levilamina()]);
    let table = chain.resolve(&[Fixtures::id("A"), Fixtures::id("B")]);
    assert_eq!(table.get(&Fixtures::id("A")).map(|resolved| resolved.value), Some(0x100));
    assert_eq!(table.source_of(&Fixtures::id("A")), Some("cache"));
    assert_eq!(table.get(&Fixtures::id("B")).map(|resolved| resolved.value), Some(0x200));
    assert_eq!(table.source_of(&Fixtures::id("B")), Some("levilamina"));
    assert!(table.missing().is_empty());
}

#[test]
fn source_order_decides_which_value_wins() {
    let dir = tempfile::tempdir().unwrap();
    let build = BuildId::new(2, 7);
    Fixtures::write_cache(
        dir.path(),
        build,
        build,
        &set(),
        vec![("A", Fixtures::resolved(EntryKind::Function, 0x100))],
    );
    let chain = ResolverChain::new(vec![levilamina(), cache(dir.path(), build)]);
    let table = chain.resolve(&[Fixtures::id("A")]);
    assert_eq!(table.get(&Fixtures::id("A")).map(|resolved| resolved.value), Some(0x999));
    assert_eq!(table.source_of(&Fixtures::id("A")), Some("levilamina"));
}

#[test]
fn missing_entries_are_listed_and_others_resolve() {
    let dir = tempfile::tempdir().unwrap();
    let build = BuildId::new(2, 7);
    Fixtures::write_cache(
        dir.path(),
        build,
        build,
        &set(),
        vec![("A", Fixtures::resolved(EntryKind::Function, 0x100))],
    );
    let table = ResolverChain::new(vec![cache(dir.path(), build)]).resolve(&[Fixtures::id("A"), Fixtures::id("B")]);
    assert_eq!(table.missing(), vec![Fixtures::id("B")]);
    assert!(table.get(&Fixtures::id("A")).is_some());
}

#[test]
fn derive_is_asked_for_a_miss() {
    let dir = tempfile::tempdir().unwrap();
    let build = BuildId::new(2, 7);
    Fixtures::write_cache(
        dir.path(),
        build,
        build,
        &set(),
        vec![("A", Fixtures::resolved(EntryKind::Function, 0x100))],
    );
    let derive = DeriveSource::new(
        dir.path().join("none.exe"),
        dir.path().join("none"),
        dir.path().join("out"),
        dir.path().join("data"),
    );
    let runs = derive.run_counter();
    let chain = ResolverChain::new(vec![cache(dir.path(), build), ResolverSource::Derive(derive)]);
    let table = chain.resolve(&[Fixtures::id("A"), Fixtures::id("B")]);
    assert_eq!(table.missing(), vec![Fixtures::id("B")]);
    assert_eq!(runs.load(Ordering::Relaxed), 1);
}

#[test]
fn derive_is_not_asked_when_every_entry_is_a_hit_or_a_known_failure() {
    let dir = tempfile::tempdir().unwrap();
    let build = BuildId::new(2, 7);
    Fixtures::write_data(
        &CacheFile::path(dir.path(), build),
        build,
        &set(),
        vec![("A", Fixtures::resolved(EntryKind::Function, 0x100))],
        vec!["B"],
    );
    let derive = DeriveSource::new(
        dir.path().join("none.exe"),
        dir.path().join("none"),
        dir.path().join("out"),
        dir.path().join("data"),
    );
    let runs = derive.run_counter();
    let chain = ResolverChain::new(vec![cache(dir.path(), build), ResolverSource::Derive(derive), levilamina()]);
    let table = chain.resolve(&[Fixtures::id("A"), Fixtures::id("B")]);
    assert_eq!(runs.load(Ordering::Relaxed), 0);
    // a known failure still goes to the sources after derive
    assert_eq!(table.source_of(&Fixtures::id("B")), Some("levilamina"));
    assert!(table.notes().is_empty());
}

#[test]
fn a_lookup_without_an_address_is_noted() {
    let chain = ResolverChain::new(vec![levilamina()]);
    let table = chain.resolve(&[Fixtures::id("C")]);
    assert_eq!(table.notes(), [SourceNote::LookupMissing(Fixtures::id("C"))]);
}

#[test]
fn derive_failures_are_noted_not_swallowed() {
    let dir = tempfile::tempdir().unwrap();
    let recipes = dir.path().join("recipes");
    std::fs::create_dir_all(&recipes).unwrap();
    let missing_exe =
        DeriveSource::new(dir.path().join("none.exe"), recipes, dir.path().join("out"), dir.path().join("data"));
    let table = ResolverChain::new(vec![ResolverSource::Derive(missing_exe)]).resolve(&[Fixtures::id("A")]);
    assert!(matches!(table.notes(), [SourceNote::ImageUnreadable(_)]));

    let missing_recipes = DeriveSource::new(
        Fixtures::probe_dll(),
        dir.path().join("nope"),
        dir.path().join("out"),
        dir.path().join("data"),
    );
    let table = ResolverChain::new(vec![ResolverSource::Derive(missing_recipes)]).resolve(&[Fixtures::id("A")]);
    assert!(matches!(table.notes(), [SourceNote::RecipesUnreadable(_)]));
}
