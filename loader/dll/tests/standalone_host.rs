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

struct Instance;

impl Instance {
    fn layout_with_log(root: &std::path::Path) -> amml::HostPaths {
        let paths = Probe::standalone_layout(root);
        std::fs::create_dir_all(&paths.logs).unwrap();
        LoaderLog::install(&paths.logs).unwrap();
        paths
    }

    fn write(paths: &amml::HostPaths, text: &str) {
        std::fs::create_dir_all(&paths.loader).unwrap();
        std::fs::write(&paths.instance, text).unwrap();
    }

    fn log_after_marker(paths: &amml::HostPaths) -> String {
        curia::info!("marker for the test");
        LogFile::wait_for(&paths.logs.join("loader.log"), "marker for the test")
    }
}

#[test]
fn install_redirect_without_instance_file_is_none() {
    let root = tempfile::tempdir().unwrap();
    let paths = Instance::layout_with_log(root.path());

    let redirect = StandaloneHost::install_redirect(&paths);

    assert!(redirect.is_none());
    let log = Instance::log_after_marker(&paths);
    assert!(!log.contains("instance.hcl"), "{log}");
}

#[test]
fn install_redirect_with_bad_file_is_none_and_logged() {
    let root = tempfile::tempdir().unwrap();
    let paths = Instance::layout_with_log(root.path());
    Instance::write(&paths, "data_dir = 3");

    let redirect = StandaloneHost::install_redirect(&paths);

    assert!(redirect.is_none());
    let log = Instance::log_after_marker(&paths);
    let lines: Vec<&str> = log.lines().filter(|line| line.contains("instance.hcl")).collect();
    assert_eq!(lines.len(), 1, "{log}");
    assert!(lines[0].contains("ERROR"), "{log}");
}

#[test]
fn install_redirect_with_data_dir_redirects_this_process_and_creates_the_folder() {
    let root = tempfile::tempdir().unwrap();
    let paths = Instance::layout_with_log(root.path());
    let data_dir = root.path().join("data");
    let escaped = data_dir.to_string_lossy().replace(std::path::MAIN_SEPARATOR, "/");
    Instance::write(&paths, &format!("data_dir = \"{escaped}\"\nfuture_key = true\n"));

    let redirect = StandaloneHost::install_redirect(&paths);

    assert!(redirect.is_some_and(|redirect| !redirect.slots().is_empty()));
    assert!(data_dir.is_dir());
    assert_eq!(std::env::var("APPDATA").unwrap(), data_dir.to_string_lossy());
    let log = Instance::log_after_marker(&paths);
    assert!(log.contains("data folder redirected"), "{log}");
    assert!(log.contains("future_key"), "{log}");
}
