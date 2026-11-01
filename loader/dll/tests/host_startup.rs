use std::path::PathBuf;

use amml::{HostInputs, HostPaths, HostStartup, LoaderLog};
use amml_data::{EntryId, EntryKind, Resolved};
use amml_runtime::{EntryTable, LayoutRefusal, ModRegistry, ModSpec};

use crate::support::LogFile;

fn spec(name: &str, entries: &[&str]) -> ModSpec {
    ModSpec {
        name: name.to_string(),
        dir: PathBuf::from(name),
        entry: format!("{name}.dll"),
        api_version: 1,
        entries: entries.iter().map(|id| EntryId(id.to_string())).collect(),
    }
}

#[test]
fn the_wanted_entries_are_the_union_over_all_mods_plus_the_game_heap() {
    let mods = [spec("a", &["x.one", "x.two"]), spec("b", &["x.two", "y.three"])];

    let wanted: Vec<String> = HostStartup::wanted(&mods).into_iter().map(|EntryId(id)| id).collect();

    assert_eq!(wanted, ["core.operator_delete", "core.operator_new", "x.one", "x.two", "y.three"]);
}

#[test]
fn without_mods_only_the_game_heap_is_wanted() {
    let wanted: Vec<String> = HostStartup::wanted(&[]).into_iter().map(|EntryId(id)| id).collect();

    assert_eq!(wanted, ["core.operator_delete", "core.operator_new"]);
}

#[test]
fn missing_mods_and_cache_folders_are_created_and_nothing_fails() {
    let dir = tempfile::tempdir().unwrap();
    let paths = HostPaths::levilamina(&dir.path().join("Minecraft.Windows.exe"));

    let errors = HostStartup::prepare_folders(&paths);

    assert!(errors.is_empty());
    assert!(paths.mods.is_dir());
    assert!(paths.cache.is_dir());
    assert!(ModRegistry::new(ModRegistry::MAX_MODS).scan(&paths.mods).unwrap().mods.is_empty());
}

#[test]
fn a_missing_recipes_folder_gives_an_empty_recipe_set_and_no_cache_key() {
    let dir = tempfile::tempdir().unwrap();
    let paths = HostPaths::levilamina(&dir.path().join("Minecraft.Windows.exe"));

    let inputs = HostInputs::read(&paths);

    assert!(inputs.set().entries().is_empty());
    assert!(inputs.cache_key().is_none());
}

#[test]
fn an_unreadable_exe_leaves_the_recipes_but_drops_cache_and_derive() {
    let dir = tempfile::tempdir().unwrap();
    let paths = HostPaths::levilamina(&dir.path().join("Minecraft.Windows.exe"));
    std::fs::create_dir_all(&paths.recipes).unwrap();

    let inputs = HostInputs::read(&paths);

    assert!(inputs.set().entries().is_empty());
    assert!(inputs.cache_key().is_none());
}

// a table of `A` (found), `R` (refused by an `entry_is` check) and, when `with_missing`, `B` (missing), logged by
// `log_coverage`; returns the loader log
fn coverage_log(with_missing: bool) -> String {
    let dir = tempfile::tempdir().unwrap();
    let logs = dir.path().join("logs");
    std::fs::create_dir_all(&logs).unwrap();
    LoaderLog::install(&logs).unwrap();
    let id = |name: &str| EntryId(name.to_string());
    let mut wanted = vec![id("A"), id("R")];
    if with_missing {
        wanted.push(id("B"));
    }
    let mut table = EntryTable::new(&wanted);
    table.insert(id("A"), Resolved { kind: EntryKind::Field, value: 8, recipes_agreeing: 2 }, "cache");
    table.refuse(id("R"), LayoutRefusal { guard: id("L"), wanted: 0, found: 1 });
    HostStartup::log_coverage(&table, wanted.len());
    LogFile::wait_for(&logs.join("loader.log"), "entry not for this build")
}

#[test]
fn an_entry_not_for_this_build_logs_at_info_and_counts_apart_from_the_resolved_line() {
    let log = coverage_log(false);
    assert!(log.contains("INFO  resolved 1/1; not for this build: 1; missing: "), "{log}");
    assert!(log.contains("INFO  entry not for this build"), "{log}");
    assert!(log.contains("\"entry\":\"R\""), "{log}");
    assert!(!log.contains("WARN"), "{log}");
}

#[test]
fn a_missing_entry_next_to_a_refused_one_breaks_the_resolved_line() {
    let log = coverage_log(true);
    assert!(log.contains("resolved 1/2; not for this build: 1; missing: B"), "{log}");
}
