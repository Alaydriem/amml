use amml::{LoaderLog, LoaderState, StandaloneHost};
use amml_data::EntryId;

use crate::support::{LogFile, Probe};

fn id(name: &str) -> EntryId {
    EntryId(name.to_string())
}

#[test]
fn entries_resolve_from_derive_then_from_the_cache_and_again_from_derive_once_the_cache_is_gone() {
    let root = tempfile::tempdir().unwrap();
    let paths = Probe::standalone_layout(root.path());
    let wanted = [id("Probe.run")];
    let rva = Some(u64::from(Probe::rva()));

    let first = StandaloneHost::resolve(&paths, &wanted);
    assert_eq!(first.source_of(&id("Probe.run")), Some("derive"));
    assert_eq!(first.get(&id("Probe.run")).map(|resolved| resolved.value), rva);

    let cached = StandaloneHost::resolve(&paths, &wanted);
    assert_eq!(cached.source_of(&id("Probe.run")), Some("cache"));
    assert_eq!(cached.get(&id("Probe.run")).map(|resolved| resolved.value), rva);

    std::fs::remove_dir_all(&paths.cache).unwrap();
    let derived = StandaloneHost::resolve(&paths, &wanted);
    assert_eq!(derived.source_of(&id("Probe.run")), Some("derive"));
    assert_eq!(derived.get(&id("Probe.run")).map(|resolved| resolved.value), rva);
}

#[test]
fn an_unreadable_exe_leaves_every_entry_missing() {
    let root = tempfile::tempdir().unwrap();
    let paths = Probe::standalone_layout(root.path());
    std::fs::write(&paths.exe, b"not a pe file").unwrap();

    let table = StandaloneHost::resolve(&paths, &[id("Probe.run")]);

    assert_eq!(table.missing(), vec![id("Probe.run")]);
}

#[test]
fn start_logs_coverage_defers_the_heap_check_and_works_once() {
    let root = tempfile::tempdir().unwrap();
    let paths = Probe::standalone_layout(root.path());
    std::fs::create_dir_all(&paths.logs).unwrap();
    LoaderLog::install(&paths.logs).unwrap();

    StandaloneHost::start(&paths, 0x1000).unwrap();

    assert!(LoaderState::get().is_some());
    assert!(paths.mods.is_dir());
    let log = LogFile::wait_for(&paths.logs.join("loader.log"), "deferred");
    assert!(log.contains("resolved 0/2"), "{log}");
    assert!(log.contains("game heap round trip deferred"), "{log}");
    assert!(!log.contains("game heap ok"), "{log}");
    assert!(StandaloneHost::start(&paths, 0x1000).is_err());
}
