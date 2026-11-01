use std::fs;

use amml::{ModLoadError, ModLoader, ModPhase};
use amml_abi::{AmmlStatus, AmmlStr};
use amml_runtime::{ModRegistry, ModSpec};

use crate::support::{EchoMod, Harness, HeapProbe, HookTarget, LogFile};

#[test]
fn mods_load_in_sorted_order_and_a_bad_manifest_skips_only_that_mod() {
    let dll = EchoMod::build("plain", &[]);
    let harness = Harness::start();
    harness.install_mod("b-second", "beta", &dll, "beta.dll", 1);
    harness.install_mod("a-first", "alpha", &dll, "alpha.dll", 1);
    fs::create_dir_all(harness.mods().join("c-broken")).unwrap();
    fs::write(harness.mods().join("c-broken").join("amml.hcl"), "name = ").unwrap();

    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    assert_eq!(scan.skipped.len(), 1);
    let loader = ModLoader::new(harness.state);
    let reports = loader.load_all(&scan.mods);

    let names: Vec<&str> = reports.iter().map(|report| report.name.as_str()).collect();
    assert_eq!(names, ["alpha", "beta"]);
    assert!(reports.iter().all(|report| report.is_ok() && report.phase == ModPhase::Loaded));
    assert_eq!(loader.phases(), [("alpha".to_string(), ModPhase::Loaded), ("beta".to_string(), ModPhase::Loaded)]);
}

#[test]
fn a_mod_needing_a_newer_api_is_refused_with_version_mismatch() {
    let newer = EchoMod::build("newer", &["newer-api"]);
    let harness = Harness::start();
    harness.install_mod("newer", "echo-newer", &newer, "echo_newer.dll", 1);
    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    let loader = ModLoader::new(harness.state);

    // The manifest says 1 but the DLL needs loader API 2, so its own load call refuses this loader.
    let mut reports = loader.load_all(&scan.mods);
    // A manifest asking for version 2 is refused before the DLL is mapped.
    let ahead = ModSpec {
        name: "ahead".to_string(),
        dir: harness.mods().join("ahead"),
        entry: "missing.dll".to_string(),
        api_version: 2,
        entries: Vec::new(),
    };
    reports.extend(loader.load_all(&[ahead]));

    for report in &reports {
        assert_eq!(report.error.as_ref().map(ModLoadError::status), Some(AmmlStatus::VERSION_MISMATCH));
    }
    assert_eq!(reports[0].phase, ModPhase::Refused);
    assert_eq!(reports[1].phase, ModPhase::Refused);
    assert!(loader.enable_all().is_empty());
}

#[test]
fn a_panicking_mod_is_disabled_and_the_next_mod_still_loads() {
    let panicking = EchoMod::build("panic", &["panic-enable"]);
    let plain = EchoMod::build("plain", &[]);
    let harness = Harness::start();
    harness.install_mod("1-panics", "echo-panic", &panicking, "echo_panic.dll", 1);
    harness.install_mod("2-echo", "echo", &plain, "echo.dll", 1);
    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    let loader = ModLoader::new(harness.state);

    let loaded = loader.load_all(&scan.mods);
    assert!(loaded.iter().all(|report| report.is_ok()));
    let enabled = loader.enable_all();

    assert_eq!(enabled[0].name, "echo-panic");
    assert_eq!(enabled[0].phase, ModPhase::Failed);
    assert_eq!(enabled[0].error.as_ref().map(ModLoadError::status), Some(AmmlStatus::PANICKED));
    assert_eq!(enabled[1].name, "echo");
    assert_eq!(enabled[1].phase, ModPhase::Enabled);
    assert!(enabled[1].is_ok());

    let echo = LogFile::wait_for(&harness.logs().join("echo.log"), "enabled");
    assert!(echo.contains(&format!("found={:#x}", Harness::BASE + 0x1234)), "{echo}");
    assert!(echo.contains("absent status=AmmlStatus(2)"), "{echo}");
    assert!(echo.contains("heap block null=false"), "{echo}");
    assert!(echo.contains(&harness.mods().join("2-echo").display().to_string()), "{echo}");
    assert_eq!(HeapProbe::counts(), (1, 1));
    let panicked = LogFile::wait_for(&harness.logs().join("echo-panic.log"), "panicked");
    assert!(panicked.contains("enable panicked: echo-mod enable panics on purpose"), "{panicked}");
    let loader_log = LogFile::wait_for(&harness.logs().join("loader.log"), "mod switched off");
    assert!(loader_log.contains("mod switched off"), "{loader_log}");

    // The panicking mod is not enabled again, and unload keeps both libraries mapped.
    loader.disable_all();
    loader.unload_all();
    assert_eq!(
        loader.phases(),
        [("echo-panic".to_string(), ModPhase::Unloaded), ("echo".to_string(), ModPhase::Unloaded)]
    );
    let echo = LogFile::wait_for(&harness.logs().join("echo.log"), "unloaded");
    assert!(echo.contains("disabled") && echo.contains("unloaded"), "{echo}");
}

#[test]
fn resolve_returns_missing_entry_for_an_unresolved_id_and_values_for_others() {
    let harness = Harness::start();
    // SAFETY: the table lives for the process and has the full v1 size.
    let api = unsafe { &*harness.state.api() };
    let resolve = |id: &str| {
        let mut out = 0u64;
        // SAFETY: the id lives for the call and `out` is a live u64.
        let status = unsafe { (api.resolve)(AmmlStr::from_str(id), &mut out) };
        (status, out)
    };

    assert_eq!(resolve("Echo.found"), (AmmlStatus::OK, Harness::BASE + 0x1234));
    assert_eq!(resolve("Echo.global"), (AmmlStatus::OK, Harness::BASE + 0x5000));
    assert_eq!(resolve("Echo.slot"), (AmmlStatus::OK, 12));
    assert_eq!(resolve("Echo.offset"), (AmmlStatus::OK, 0x30));
    assert_eq!(resolve("Echo.absent").0, AmmlStatus::MISSING_ENTRY);
    assert_eq!(resolve("Never.heard.of").0, AmmlStatus::MISSING_ENTRY);
}

#[test]
fn config_and_data_folders_are_inside_the_mod_folder() {
    let dll = EchoMod::build("plain", &[]);
    let harness = Harness::start();
    harness.install_mod("alpha-dir", "alpha", &dll, "alpha.dll", 1);
    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    ModLoader::new(harness.state).load_all(&scan.mods);

    // SAFETY: the table lives for the process and has the full v1 size.
    let api = unsafe { &*harness.state.api() };
    let dir = |kind: u8| {
        let mut out = AmmlStr::EMPTY;
        // SAFETY: `out` is a writable AmmlStr and the name is a live view.
        let status = unsafe { (api.mod_dir)(AmmlStr::from_str("alpha"), kind, &mut out) };
        assert_eq!(status, AmmlStatus::OK);
        // SAFETY: the loader keeps the bytes valid for the life of the process.
        std::path::PathBuf::from(unsafe { out.as_str() }.unwrap())
    };
    let mod_dir = harness.mods().join("alpha-dir");

    assert_eq!(dir(0), mod_dir);
    assert_eq!(dir(1), mod_dir.join("config"));
    assert_eq!(dir(2), mod_dir.join("data"));
}

#[test]
fn disable_all_removes_the_hooks_of_a_mod_and_a_hook_from_disable_is_refused() {
    let dll = EchoMod::build("hook", &["hook-target"]);
    let harness = Harness::start();
    harness.install_mod("hook", "echo-hook", &dll, "echo_hook.dll", 1);
    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    let loader = ModLoader::new(harness.state);
    loader.load_all(&scan.mods);
    assert_eq!(HookTarget::call(1), 2);

    let enabled = loader.enable_all();
    assert!(enabled.iter().all(|report| report.is_ok()));
    LogFile::wait_for(&harness.logs().join("echo-hook.log"), "hooked handle=");

    // target(1) = 2, and the mod adds 1000
    assert_eq!(HookTarget::call(1), 1002);

    loader.disable_all();

    assert_eq!(HookTarget::call(1), 2);
    assert_eq!(loader.phases(), [("echo-hook".to_string(), ModPhase::Disabled)]);
    let log = LogFile::wait_for(&harness.logs().join("echo-hook.log"), "disable hook");
    assert!(log.contains(&format!("disable hook status={:?}", AmmlStatus::FAILED)), "{log}");
    assert!(!log.contains("disable hook accepted"), "{log}");
}

#[test]
fn a_mod_cannot_remove_the_hook_of_another_mod() {
    let hooking = EchoMod::build("hook", &["hook-target"]);
    let thief = EchoMod::build("thief", &["steal-hook"]);
    let harness = Harness::start();
    harness.install_mod("1-hook", "echo-hook", &hooking, "echo_hook.dll", 1);
    harness.install_mod("2-thief", "echo-thief", &thief, "echo_thief.dll", 1);
    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    let loader = ModLoader::new(harness.state);
    loader.load_all(&scan.mods);

    let enabled = loader.enable_all();

    assert!(enabled.iter().all(|report| report.is_ok()));
    let log = LogFile::wait_for(&harness.logs().join("echo-thief.log"), "unhook");
    assert!(log.contains(&format!("unhook status={:?}", AmmlStatus::FAILED)), "{log}");
    assert_eq!(HookTarget::call(1), 1002);

    loader.disable_all();

    assert_eq!(HookTarget::call(1), 2);
}
