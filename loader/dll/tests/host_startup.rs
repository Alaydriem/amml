use std::path::PathBuf;

use amml::{HostInputs, HostPaths, HostStartup, LoaderLog};
use amml_data::{EntryId, EntryKind, Resolved};
use amml_runtime::{EntryTable, LayoutRefusal, ModRegistry, ModSpec};

use crate::support::LogFile;

fn spec(name: &str, entries: &[&str]) -> ModSpec {
    ModSpec {
        namespace: "acme".to_string(),
        name: name.to_string(),
        dir: PathBuf::from(name),
        entry: format!("{name}.dll"),
        api_version: 1,
        version: None,
        amml_version: None,
        entries: entries.iter().map(|id| EntryId(id.to_string())).collect(),
        ignored: Vec::new(),
    }
}

#[test]
fn the_wanted_entries_are_the_union_over_all_mods_plus_the_loader_entries() {
    let mods = [spec("a", &["x.one", "x.two"]), spec("b", &["x.two", "y.three"])];

    let wanted: Vec<String> = HostStartup::wanted(&mods).into_iter().map(|EntryId(id)| id).collect();

    assert_eq!(
        wanted,
        [
            "MinecraftScreenModel.getVersionString",
            "core.operator_delete",
            "core.operator_new",
            "x.one",
            "x.two",
            "y.three"
        ]
    );
}

#[test]
fn without_mods_only_the_loader_entries_are_wanted() {
    let wanted: Vec<String> = HostStartup::wanted(&[]).into_iter().map(|EntryId(id)| id).collect();

    assert_eq!(wanted, ["MinecraftScreenModel.getVersionString", "core.operator_delete", "core.operator_new"]);
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
fn cache_files_of_the_older_naming_are_removed_at_start() {
    let dir = tempfile::tempdir().unwrap();
    let paths = HostPaths::levilamina(&dir.path().join("Minecraft.Windows.exe"));
    std::fs::create_dir_all(&paths.cache).unwrap();
    let stale = paths.cache.join("6aa482fd-2658118eba0dbac5-0123456789abcdef.hcl");
    let current = paths.cache.join("6aa482fd-2658118eba0dbac5.hcl");
    std::fs::write(&stale, "").unwrap();
    std::fs::write(&current, "").unwrap();

    let errors = HostStartup::prepare_folders(&paths);

    assert!(errors.is_empty());
    assert!(!stale.exists());
    assert!(current.exists());
}

#[test]
fn a_missing_recipes_folder_gives_an_empty_recipe_set_and_no_build() {
    let dir = tempfile::tempdir().unwrap();
    let paths = HostPaths::levilamina(&dir.path().join("Minecraft.Windows.exe"));

    let inputs = HostInputs::read(&paths);

    assert!(inputs.set().entries().is_empty());
    assert!(inputs.build().is_none());
}

#[test]
fn an_unreadable_exe_leaves_the_recipes_but_drops_cache_and_derive() {
    let dir = tempfile::tempdir().unwrap();
    let paths = HostPaths::levilamina(&dir.path().join("Minecraft.Windows.exe"));
    std::fs::create_dir_all(&paths.recipes).unwrap();

    let inputs = HostInputs::read(&paths);

    assert!(inputs.set().entries().is_empty());
    assert!(inputs.build().is_none());
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

#[test]
fn the_derive_summary_and_a_missing_cache_log_at_info_and_other_notes_at_warn() {
    let dir = tempfile::tempdir().unwrap();
    let logs = dir.path().join("logs");
    std::fs::create_dir_all(&logs).unwrap();
    LoaderLog::install(&logs).unwrap();
    let mut table = EntryTable::new(&[]);
    table.add_notes(vec![
        amml_runtime::SourceNote::CacheMissing,
        amml_runtime::SourceNote::DeriveRan { hits: 3, derived: 2, elapsed_ms: 41 },
        amml_runtime::SourceNote::DeriveFailed("boom".to_string()),
    ]);

    HostStartup::log_coverage(&table, 0);

    let log = LogFile::wait_for(&logs.join("loader.log"), "boom");
    assert!(log.contains("INFO  entries resolved from cache and derive"), "{log}");
    assert!(log.contains("\"hits\":3") && log.contains("\"derived\":2") && log.contains("\"ms\":41"), "{log}");
    assert!(log.contains("INFO  resolver note"), "{log}");
    assert!(log.contains("WARN  resolver note"), "{log}");
    assert_eq!(log.matches("WARN").count(), 1, "{log}");
    assert_eq!(log.matches("entries resolved from cache and derive").count(), 1, "{log}");
}

#[test]
fn a_start_served_fully_from_the_cache_logs_the_summary_with_nothing_derived() {
    let dir = tempfile::tempdir().unwrap();
    let logs = dir.path().join("logs");
    std::fs::create_dir_all(&logs).unwrap();
    LoaderLog::install(&logs).unwrap();
    let id = |name: &str| EntryId(name.to_string());
    let mut table = EntryTable::new(&[id("A"), id("B")]);
    table.insert(id("A"), Resolved { kind: EntryKind::Field, value: 8, recipes_agreeing: 2 }, "cache");
    table.insert(id("B"), Resolved { kind: EntryKind::Field, value: 16, recipes_agreeing: 2 }, "cache");

    HostStartup::log_coverage(&table, 2);

    let log = LogFile::wait_for(&logs.join("loader.log"), "entries resolved from cache and derive");
    assert!(log.contains("INFO  entries resolved from cache and derive"), "{log}");
    assert!(log.contains("\"hits\":2") && log.contains("\"derived\":0") && log.contains("\"ms\":0"), "{log}");
}

#[test]
fn an_entry_from_levilamina_is_not_counted_as_a_cache_hit() {
    let dir = tempfile::tempdir().unwrap();
    let logs = dir.path().join("logs");
    std::fs::create_dir_all(&logs).unwrap();
    LoaderLog::install(&logs).unwrap();
    let id = |name: &str| EntryId(name.to_string());
    let mut table = EntryTable::new(&[id("A"), id("B")]);
    table.insert(id("A"), Resolved { kind: EntryKind::Field, value: 8, recipes_agreeing: 2 }, "cache");
    table.insert(id("B"), Resolved { kind: EntryKind::Field, value: 16, recipes_agreeing: 2 }, "levilamina");

    HostStartup::log_coverage(&table, 2);

    let log = LogFile::wait_for(&logs.join("loader.log"), "entries resolved from cache and derive");
    assert!(log.contains("\"hits\":1") && log.contains("\"derived\":0"), "{log}");
}
