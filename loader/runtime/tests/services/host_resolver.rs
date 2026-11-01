use std::sync::atomic::Ordering;

use amml_data::{BuildId, EntryKind, RecipeLoader};
use amml_runtime::{CacheSource, DeriveSource, HostResolver, LayoutRefusal, LeviLaminaSource, SourceNote};

use crate::support::{BASE, Fixtures};

const DIGEST: &str = "d1";

fn lookup(name: &str) -> Option<u64> {
    match name {
        "A" | "G" => Some(BASE + 0x999),
        _ => None,
    }
}

// agrees with the cache on the function
fn agreeing_lookup(name: &str) -> Option<u64> {
    match name {
        "A" => Some(BASE + 0x100),
        _ => None,
    }
}

fn no_symbols(_name: &str) -> Option<u64> {
    None
}

struct Setup {
    // keeps the data directory alive
    _dir: tempfile::TempDir,
    cache: CacheSource,
    derive: DeriveSource,
    set: amml_data::RecipeSet,
}

// a cache holding A (function) and G (global) with values that differ from the fake lookup
fn setup() -> Setup {
    let dir = tempfile::tempdir().unwrap();
    let set = Fixtures::set(vec![
        Fixtures::entry("A", EntryKind::Function, Some("A")),
        Fixtures::entry("G", EntryKind::Global, Some("G")),
    ]);
    let build = BuildId::new(2, 7);
    Fixtures::write_cache(
        dir.path(),
        build,
        DIGEST,
        build,
        vec![
            ("A", Fixtures::resolved(EntryKind::Function, 0x100)),
            ("G", Fixtures::resolved(EntryKind::Global, 0x101)),
        ],
    );
    let cache = CacheSource::new(dir.path().to_path_buf(), build, DIGEST.to_string(), set.clone());
    let derive = DeriveSource::new(dir.path().join("none.exe"), dir.path().join("none"), dir.path().join("out"));
    Setup { _dir: dir, cache, derive, set }
}

#[test]
fn a_function_levilamina_and_the_cache_disagree_on_is_missing_and_other_kinds_prefer_the_cache() {
    let Setup { _dir, cache, derive, set } = setup();
    let levilamina = LeviLaminaSource::new(lookup, set.clone(), BASE, Fixtures::layout());
    let table =
        HostResolver::levilamina(cache, derive, levilamina, set).resolve(&[Fixtures::id("A"), Fixtures::id("G")]);
    assert_eq!(table.get(&Fixtures::id("A")), None);
    assert_eq!(table.missing(), vec![Fixtures::id("A")]);
    assert!(table.notes().contains(&SourceNote::SourcesDisagree {
        id: Fixtures::id("A"),
        levilamina: 0x999,
        derived: 0x100
    }));
    assert_eq!(table.get(&Fixtures::id("G")).map(|resolved| resolved.value), Some(0x101));
    assert_eq!(table.source_of(&Fixtures::id("G")), Some("cache"));
}

#[test]
fn a_function_levilamina_and_the_cache_agree_on_is_resolved() {
    let Setup { _dir, cache, derive, set } = setup();
    let levilamina = LeviLaminaSource::new(agreeing_lookup, set.clone(), BASE, Fixtures::layout());
    let table = HostResolver::levilamina(cache, derive, levilamina, set).resolve(&[Fixtures::id("A")]);
    assert_eq!(table.get(&Fixtures::id("A")).map(|resolved| resolved.value), Some(0x100));
    assert_eq!(table.source_of(&Fixtures::id("A")), Some("levilamina"));
    assert!(table.notes().is_empty());
}

#[test]
fn on_the_standalone_host_the_cache_wins_for_every_kind() {
    let Setup { _dir, cache, derive, set } = setup();
    let table = HostResolver::standalone(cache, derive, set).resolve(&[Fixtures::id("A"), Fixtures::id("G")]);
    assert_eq!(table.get(&Fixtures::id("A")).map(|resolved| resolved.value), Some(0x100));
    assert_eq!(table.source_of(&Fixtures::id("G")), Some("cache"));
}

#[test]
fn derive_runs_once_per_resolve_and_serves_both_orders() {
    let dir = tempfile::tempdir().unwrap();
    let probe = Fixtures::probe_dir(dir.path());
    let (host, runs) = Fixtures::probe_levilamina(&probe, no_symbols);
    let wanted = [Fixtures::id("Probe.run"), Fixtures::id("Probe.global")];

    let table = host.resolve(&wanted);
    assert_eq!(runs.load(Ordering::Relaxed), 1);
    assert_eq!(table.source_of(&Fixtures::id("Probe.run")), Some("derive"));
    assert_eq!(table.source_of(&Fixtures::id("Probe.global")), Some("derive"));
    assert_eq!(table.get(&Fixtures::id("Probe.run")).map(|resolved| resolved.value), Some(u64::from(probe.rva)));
    assert!(table.notes().contains(&SourceNote::LookupMissing(Fixtures::id("Probe.run"))));

    let again = host.resolve(&wanted);
    assert_eq!(runs.load(Ordering::Relaxed), 1);
    assert_eq!(again.source_of(&Fixtures::id("Probe.run")), Some("cache"));
}

#[test]
fn a_partial_derive_writes_a_file_and_later_resolves_do_not_derive_again() {
    let dir = tempfile::tempdir().unwrap();
    let probe = Fixtures::probe_dir(dir.path());
    let (host, runs) = Fixtures::probe_standalone(&probe);
    let wanted = [Fixtures::id("Probe.run"), Fixtures::id("Probe.absent")];

    let first = host.resolve(&wanted);
    assert_eq!(runs.load(Ordering::Relaxed), 1);
    assert_eq!(first.missing(), vec![Fixtures::id("Probe.absent")]);

    let second = host.resolve(&wanted);
    assert_eq!(runs.load(Ordering::Relaxed), 1);
    assert_eq!(second.missing(), vec![Fixtures::id("Probe.absent")]);
    assert_eq!(second.source_of(&Fixtures::id("Probe.run")), Some("cache"));
}

#[test]
fn a_changed_recipe_file_makes_the_old_cache_unreadable_and_derives_again() {
    let dir = tempfile::tempdir().unwrap();
    let probe = Fixtures::probe_dir(dir.path());
    let wanted = [Fixtures::id("Probe.run")];
    let (host, runs) = Fixtures::probe_standalone(&probe);
    host.resolve(&wanted);
    assert_eq!(runs.load(Ordering::Relaxed), 1);

    let file = probe.recipes.join("probe.hcl");
    let text = std::fs::read_to_string(&file).unwrap();
    std::fs::write(&file, format!("{text}\n# edited\n")).unwrap();
    assert!(RecipeLoader::load_dir(&probe.recipes).is_ok());

    let (changed, changed_runs) = Fixtures::probe_standalone(&probe);
    let table = changed.resolve(&wanted);
    assert_eq!(changed_runs.load(Ordering::Relaxed), 1);
    assert_eq!(table.source_of(&Fixtures::id("Probe.run")), Some("derive"));
    assert_eq!(table.notes().first(), Some(&SourceNote::CacheMissing));
}

#[test]
fn levilamina_only_resolves_functions_and_globals_from_levilamina() {
    let Setup { _dir, set, .. } = setup();
    let levilamina = LeviLaminaSource::new(lookup, set.clone(), BASE, Fixtures::layout());
    let table = HostResolver::levilamina_only(levilamina, set).resolve(&[Fixtures::id("A"), Fixtures::id("G")]);
    assert_eq!(table.get(&Fixtures::id("A")).map(|resolved| resolved.value), Some(0x999));
    assert_eq!(table.source_of(&Fixtures::id("G")), Some("levilamina"));
    assert!(table.missing().is_empty());
}

// a cache on build 2.7 holding the layout guard `L` (value 1) and the field `V`; `P` (guarded by `L = 0`) and `M`
// (guarded by `L = 1`) have no value, `Q` is guarded by `U`, which has no value either
fn guarded_setup() -> (tempfile::TempDir, HostResolver) {
    let dir = tempfile::tempdir().unwrap();
    let guarded = |name: &str, guard: &str, value: u64| {
        let mut entry = Fixtures::entry(name, EntryKind::Field, None);
        entry.checks = vec![amml_data::Check::EntryIs { entry: Fixtures::id(guard), value }];
        entry
    };
    let set = Fixtures::set(vec![
        Fixtures::entry("L", EntryKind::Field, None),
        Fixtures::entry("U", EntryKind::Field, None),
        guarded("V", "L", 1),
        guarded("P", "L", 0),
        guarded("M", "L", 1),
        guarded("Q", "U", 0),
    ]);
    let build = BuildId::new(2, 7);
    Fixtures::write_cache(
        dir.path(),
        build,
        DIGEST,
        build,
        vec![("L", Fixtures::resolved(EntryKind::Field, 1)), ("V", Fixtures::resolved(EntryKind::Field, 0x28))],
    );
    let cache = CacheSource::new(dir.path().to_path_buf(), build, DIGEST.to_string(), set.clone());
    let derive = DeriveSource::new(dir.path().join("none.exe"), dir.path().join("none"), dir.path().join("out"));
    (dir, HostResolver::standalone(cache, derive, set))
}

#[test]
fn an_entry_its_entry_is_check_refuses_is_not_for_this_build_and_not_missing() {
    let (_dir, host) = guarded_setup();
    let table = host.resolve(&[Fixtures::id("V"), Fixtures::id("P")]);
    assert!(table.missing().is_empty());
    assert_eq!(table.get(&Fixtures::id("P")), None);
    assert_eq!(
        table.not_for_build().get(&Fixtures::id("P")),
        Some(&LayoutRefusal { guard: Fixtures::id("L"), wanted: 0, found: 1 })
    );
    // the guard was resolved for the judgement only
    assert_eq!(table.get(&Fixtures::id("L")), None);
    assert_eq!(table.get(&Fixtures::id("V")).map(|resolved| resolved.value), Some(0x28));
}

#[test]
fn an_entry_whose_guard_holds_or_has_no_value_is_missing() {
    let (_dir, host) = guarded_setup();
    let table = host.resolve(&[Fixtures::id("M"), Fixtures::id("Q"), Fixtures::id("P")]);
    assert_eq!(table.missing(), vec![Fixtures::id("M"), Fixtures::id("Q")]);
    assert_eq!(table.not_for_build().keys().cloned().collect::<Vec<_>>(), vec![Fixtures::id("P")]);
}
