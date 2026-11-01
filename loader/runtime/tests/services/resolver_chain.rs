use std::sync::atomic::Ordering;

use amml_data::{BuildId, EntryKind};
use amml_runtime::{CacheSource, DeriveSource, LeviLaminaSource, ResolverChain, ResolverSource, SourceNote};

use crate::support::{BASE, Fixtures};

const DIGEST: &str = "d1";

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
    ResolverSource::Cache(CacheSource::new(dir.to_path_buf(), build, DIGEST.to_string(), set()))
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
        DIGEST,
        old,
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
fn a_cache_for_other_recipes_is_not_read() {
    let dir = tempfile::tempdir().unwrap();
    let build = BuildId::new(2, 7);
    Fixtures::write_cache(dir.path(), build, "other", build, vec![("A", Fixtures::resolved(EntryKind::Function, 1))]);
    let table = ResolverChain::new(vec![cache(dir.path(), build)]).resolve(&[Fixtures::id("A")]);
    assert_eq!(table.get(&Fixtures::id("A")), None);
    assert_eq!(table.notes(), [SourceNote::CacheMissing]);
}

#[test]
fn cache_entries_with_the_wrong_kind_or_outside_the_set_are_dropped() {
    let dir = tempfile::tempdir().unwrap();
    let build = BuildId::new(2, 7);
    Fixtures::write_cache(
        dir.path(),
        build,
        DIGEST,
        build,
        vec![
            ("A", Fixtures::resolved(EntryKind::Global, 0x100)),
            ("B", Fixtures::resolved(EntryKind::Function, 0x200)),
            ("Unknown", Fixtures::resolved(EntryKind::Function, 0x300)),
        ],
    );
    let wanted = [Fixtures::id("A"), Fixtures::id("B"), Fixtures::id("Unknown")];
    let table = ResolverChain::new(vec![cache(dir.path(), build)]).resolve(&wanted);
    assert_eq!(table.missing(), vec![Fixtures::id("A"), Fixtures::id("Unknown")]);
    assert_eq!(table.get(&Fixtures::id("B")).map(|resolved| resolved.value), Some(0x200));
    assert_eq!(table.notes(), [SourceNote::CacheKindMismatch(Fixtures::id("A"))]);
}

#[test]
fn later_sources_fill_only_what_earlier_ones_missed() {
    let dir = tempfile::tempdir().unwrap();
    let build = BuildId::new(2, 7);
    Fixtures::write_cache(
        dir.path(),
        build,
        DIGEST,
        build,
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
        DIGEST,
        build,
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
        DIGEST,
        build,
        vec![("A", Fixtures::resolved(EntryKind::Function, 0x100))],
    );
    let table = ResolverChain::new(vec![cache(dir.path(), build)]).resolve(&[Fixtures::id("A"), Fixtures::id("B")]);
    assert_eq!(table.missing(), vec![Fixtures::id("B")]);
    assert!(table.get(&Fixtures::id("A")).is_some());
}

#[test]
fn a_valid_cache_file_is_final_and_derive_is_not_asked() {
    let dir = tempfile::tempdir().unwrap();
    let build = BuildId::new(2, 7);
    Fixtures::write_cache(
        dir.path(),
        build,
        DIGEST,
        build,
        vec![("A", Fixtures::resolved(EntryKind::Function, 0x100))],
    );
    let derive = DeriveSource::new(dir.path().join("none.exe"), dir.path().join("none"), dir.path().join("out"));
    let runs = derive.run_counter();
    let chain = ResolverChain::new(vec![cache(dir.path(), build), ResolverSource::Derive(derive)]);
    let table = chain.resolve(&[Fixtures::id("A"), Fixtures::id("B")]);
    assert_eq!(table.missing(), vec![Fixtures::id("B")]);
    assert_eq!(runs.load(Ordering::Relaxed), 0);
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
    let missing_exe = DeriveSource::new(dir.path().join("none.exe"), recipes, dir.path().join("out"));
    let table = ResolverChain::new(vec![ResolverSource::Derive(missing_exe)]).resolve(&[Fixtures::id("A")]);
    assert!(matches!(table.notes(), [SourceNote::ImageUnreadable(_)]));

    let missing_recipes = DeriveSource::new(Fixtures::probe_dll(), dir.path().join("nope"), dir.path().join("out"));
    let table = ResolverChain::new(vec![ResolverSource::Derive(missing_recipes)]).resolve(&[Fixtures::id("A")]);
    assert!(matches!(table.notes(), [SourceNote::RecipesUnreadable(_)]));
}
