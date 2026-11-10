use std::fs;

use amml::{ModLoadError, ModLoader, ModPhase};
use amml_abi::{AmmlStatus, AmmlStr};
use amml_runtime::{AmmlVersion, ModRegistry, ModSpec};

use crate::support::{EchoMod, Harness, HeapProbe, HookTarget, LogFile};

#[test]
fn mods_load_in_sorted_order_and_a_bad_manifest_skips_only_that_mod() {
    let plain = EchoMod::build("plain", &[]);
    let hooking = EchoMod::build("hook", &["hook-target"]);
    let harness = Harness::start();
    harness.install_mod("echo", "echo-hook", &hooking, "echo_hook.dll", 2);
    harness.install_mod("echo", "echo", &plain, "echo.dll", 2);
    fs::create_dir_all(harness.mods().join("@echo").join("c-broken")).unwrap();
    fs::write(harness.mods().join("@echo").join("c-broken").join("amml.hcl"), "name = ").unwrap();

    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    assert_eq!(scan.skipped.len(), 1);
    let loader = ModLoader::new(harness.state);
    let reports = loader.load_all(&scan.mods);

    let ids: Vec<&str> = reports.iter().map(|report| report.id.as_str()).collect();
    assert_eq!(ids, ["@echo/echo", "@echo/echo-hook"]);
    assert!(reports.iter().all(|report| report.is_ok() && report.phase == ModPhase::Loaded));
    assert_eq!(
        loader.phases(),
        [("@echo/echo".to_string(), ModPhase::Loaded), ("@echo/echo-hook".to_string(), ModPhase::Loaded)]
    );
}

#[test]
fn a_mod_that_reports_another_namespace_is_refused() {
    let plain = EchoMod::build("plain", &[]);
    let hooking = EchoMod::build("hook", &["hook-target"]);
    let harness = Harness::start();
    harness.install_mod("other", "echo", &plain, "echo.dll", 2);
    harness.install_mod("echo", "echo-hook", &hooking, "echo_hook.dll", 2);
    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    let loader = ModLoader::new(harness.state);

    let reports = loader.load_all(&scan.mods);

    assert_eq!(reports[0].id, "@echo/echo-hook");
    assert!(reports[0].is_ok());
    assert_eq!(reports[1].id, "@other/echo");
    assert!(matches!(&reports[1].error, Some(ModLoadError::IdMismatch { manifest, reported })
            if manifest == "@other/echo" && reported == "@echo/echo"));
    assert_eq!(reports[1].phase, ModPhase::Failed);
    assert!(loader.enable_all().iter().all(|report| report.id == "@echo/echo-hook" && report.is_ok()));
}

#[test]
fn a_v1_info_block_is_refused_as_built_for_an_older_loader() {
    let old = EchoMod::build("old", &["old-info"]);
    let plain = EchoMod::build("plain", &[]);
    let harness = Harness::start();
    harness.install_mod("echo", "echo-old", &old, "echo_old.dll", 1);
    harness.install_mod("echo", "echo", &plain, "echo.dll", 2);
    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    let loader = ModLoader::new(harness.state);

    let reports = loader.load_all(&scan.mods);

    assert_eq!(reports[0].id, "@echo/echo");
    assert!(reports[0].is_ok());
    assert!(matches!(reports[1].error, Some(ModLoadError::OlderMod { .. })));
    assert_eq!(reports[1].error.as_ref().map(ModLoadError::status), Some(AmmlStatus::VERSION_MISMATCH));
    assert_eq!(reports[1].phase, ModPhase::Refused);
}

#[test]
fn a_mod_needing_a_newer_api_is_refused_with_version_mismatch() {
    let newer = EchoMod::build("newer", &["newer-api"]);
    let harness = Harness::start();
    harness.install_mod("echo", "echo-newer", &newer, "echo_newer.dll", 2);
    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    let loader = ModLoader::new(harness.state);

    // The manifest says 2 but the DLL needs loader API 3, so its own load call refuses this loader.
    let mut reports = loader.load_all(&scan.mods);
    // A manifest asking for version 3 is refused before the DLL is mapped.
    let ahead = ModSpec {
        namespace: "echo".to_string(),
        name: "ahead".to_string(),
        dir: harness.mods().join("@echo").join("ahead"),
        entry: "missing.dll".to_string(),
        api_version: 3,
        version: None,
        amml_version: None,
        entries: Vec::new(),
        ignored: Vec::new(),
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
    harness.install_mod("echo", "echo-panic", &panicking, "echo_panic.dll", 2);
    harness.install_mod("echo", "echo", &plain, "echo.dll", 2);
    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    let loader = ModLoader::new(harness.state);

    let loaded = loader.load_all(&scan.mods);
    assert!(loaded.iter().all(|report| report.is_ok()));
    let enabled = loader.enable_all();

    assert_eq!(enabled[1].id, "@echo/echo-panic");
    assert_eq!(enabled[1].phase, ModPhase::Failed);
    assert_eq!(enabled[1].error.as_ref().map(ModLoadError::status), Some(AmmlStatus::PANICKED));
    assert_eq!(enabled[0].id, "@echo/echo");
    assert_eq!(enabled[0].phase, ModPhase::Enabled);
    assert!(enabled[0].is_ok());

    let echo = LogFile::wait_for(&harness.logs().join("@echo").join("echo.log"), "enabled");
    assert!(echo.contains(&format!("found={:#x}", Harness::BASE + 0x1234)), "{echo}");
    assert!(echo.contains("absent status=AmmlStatus(2)"), "{echo}");
    assert!(echo.contains("heap block null=false"), "{echo}");
    assert!(echo.contains(&harness.mods().join("@echo").join("echo").display().to_string()), "{echo}");
    assert_eq!(HeapProbe::counts(), (1, 1));
    let panicked = LogFile::wait_for(&harness.logs().join("@echo").join("echo-panic.log"), "panicked");
    assert!(panicked.contains("enable panicked: echo-mod enable panics on purpose"), "{panicked}");
    let loader_log = LogFile::wait_for(&harness.logs().join("loader.log"), "mod switched off");
    assert!(loader_log.contains("mod switched off"), "{loader_log}");

    // The panicking mod is not enabled again, and unload keeps both libraries mapped.
    loader.disable_all();
    loader.unload_all();
    assert_eq!(
        loader.phases(),
        [("@echo/echo".to_string(), ModPhase::Unloaded), ("@echo/echo-panic".to_string(), ModPhase::Unloaded)]
    );
    let echo = LogFile::wait_for(&harness.logs().join("@echo").join("echo.log"), "unloaded");
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
    harness.install_mod("echo", "echo", &dll, "echo.dll", 2);
    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    ModLoader::new(harness.state).load_all(&scan.mods);

    // SAFETY: the table lives for the process and has the full v1 size.
    let api = unsafe { &*harness.state.api() };
    let dir = |kind: u8| {
        let mut out = AmmlStr::EMPTY;
        // SAFETY: `out` is a writable AmmlStr and the name is a live view.
        let status = unsafe { (api.mod_dir)(AmmlStr::from_str("@echo/echo"), kind, &mut out) };
        assert_eq!(status, AmmlStatus::OK);
        // SAFETY: the loader keeps the bytes valid for the life of the process.
        std::path::PathBuf::from(unsafe { out.as_str() }.unwrap())
    };
    let mod_dir = harness.mods().join("@echo").join("echo");

    assert_eq!(dir(0), mod_dir);
    assert_eq!(dir(1), mod_dir.join("config"));
    assert_eq!(dir(2), mod_dir.join("data"));
    assert_eq!(dir(3), harness.mod_data().join("@echo").join("echo"));
}

#[test]
fn without_a_shared_data_root_kind_3_is_refused_and_the_mod_still_loads() {
    let dll = EchoMod::build("plain", &[]);
    let harness = Harness::start_with(false);
    harness.install_mod("echo", "echo", &dll, "echo.dll", 2);
    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    ModLoader::new(harness.state).load_all(&scan.mods);

    // SAFETY: the table lives for the process and has the full v1 size.
    let api = unsafe { &*harness.state.api() };
    let status = |kind: u8| {
        let mut out = AmmlStr::EMPTY;
        // SAFETY: `out` is a writable AmmlStr and the name is a live view.
        unsafe { (api.mod_dir)(AmmlStr::from_str("@echo/echo"), kind, &mut out) }
    };

    assert_eq!(status(0), AmmlStatus::OK);
    assert_eq!(status(1), AmmlStatus::OK);
    assert_eq!(status(2), AmmlStatus::OK);
    assert_eq!(status(3), AmmlStatus::INVALID_ARGUMENT);
}

#[test]
fn disable_all_removes_the_hooks_of_a_mod_and_a_hook_from_disable_is_refused() {
    let dll = EchoMod::build("hook", &["hook-target"]);
    let harness = Harness::start();
    harness.install_mod("echo", "echo-hook", &dll, "echo_hook.dll", 2);
    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    let loader = ModLoader::new(harness.state);
    loader.load_all(&scan.mods);
    assert_eq!(HookTarget::call(1), 2);

    let enabled = loader.enable_all();
    assert!(enabled.iter().all(|report| report.is_ok()));
    LogFile::wait_for(&harness.logs().join("@echo").join("echo-hook.log"), "hooked handle=");

    // target(1) = 2, and the mod adds 1000
    assert_eq!(HookTarget::call(1), 1002);

    loader.disable_all();

    assert_eq!(HookTarget::call(1), 2);
    assert_eq!(loader.phases(), [("@echo/echo-hook".to_string(), ModPhase::Disabled)]);
    let log = LogFile::wait_for(&harness.logs().join("@echo").join("echo-hook.log"), "disable hook");
    assert!(log.contains(&format!("disable hook status={:?}", AmmlStatus::FAILED)), "{log}");
    assert!(!log.contains("disable hook accepted"), "{log}");
}

#[test]
fn a_mod_cannot_remove_the_hook_of_another_mod() {
    let hooking = EchoMod::build("hook", &["hook-target"]);
    let thief = EchoMod::build("thief", &["steal-hook"]);
    let harness = Harness::start();
    harness.install_mod("echo", "echo-hook", &hooking, "echo_hook.dll", 2);
    harness.install_mod("echo", "echo-thief", &thief, "echo_thief.dll", 2);
    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    let loader = ModLoader::new(harness.state);
    loader.load_all(&scan.mods);

    let enabled = loader.enable_all();

    assert!(enabled.iter().all(|report| report.is_ok()));
    let log = LogFile::wait_for(&harness.logs().join("@echo").join("echo-thief.log"), "unhook");
    assert!(log.contains(&format!("unhook status={:?}", AmmlStatus::FAILED)), "{log}");
    assert_eq!(HookTarget::call(1), 1002);

    loader.disable_all();

    assert_eq!(HookTarget::call(1), 2);
}

fn spec_needing(harness: &Harness, name: &str, needs: AmmlVersion) -> ModSpec {
    ModSpec {
        namespace: "echo".to_string(),
        name: name.to_string(),
        dir: harness.mods().join("@echo").join(name),
        entry: "missing.dll".to_string(),
        api_version: 2,
        version: None,
        amml_version: Some(needs),
        entries: Vec::new(),
        ignored: Vec::new(),
    }
}

#[test]
fn refuses_mod_needing_newer_amml() {
    let plain = EchoMod::build("plain", &[]);
    let harness = Harness::start();
    harness.install_mod("echo", "echo", &plain, "echo.dll", 2);
    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    let loader = ModLoader::new(harness.state);
    let future = AmmlVersion { major: 99, minor: 0, patch: 0 };
    let mut specs = vec![spec_needing(&harness, "future", future)];
    specs.extend(scan.mods);

    let reports = loader.load_all(&specs);

    assert_eq!(reports[0].phase, ModPhase::Refused);
    assert!(matches!(reports[0].error, Some(ModLoadError::AmmlTooOld { needs, .. }) if needs == future));
    assert_eq!(reports[0].error.as_ref().map(ModLoadError::status), Some(AmmlStatus::VERSION_MISMATCH));
    assert_eq!(reports[1].phase, ModPhase::Loaded);
    assert!(reports[1].is_ok());
}

#[test]
fn loads_mod_at_exact_amml_version() {
    let harness = Harness::start();
    let loader = ModLoader::new(harness.state);
    let own = ModLoader::amml_version();

    let reports = loader.load_all(&[spec_needing(&harness, "exact", own)]);

    // The AMML check passes; the missing DLL then fails the library step, not the version step.
    assert!(!matches!(reports[0].error, Some(ModLoadError::AmmlTooOld { .. })));
    assert!(matches!(reports[0].error, Some(ModLoadError::Library { .. })));
}

#[test]
fn loader_version_matches_the_package_version() {
    assert_eq!(Some(ModLoader::amml_version()), AmmlVersion::parse(env!("CARGO_PKG_VERSION")));
}

#[test]
fn folder_without_amml_version_loads() {
    let plain = EchoMod::build("plain", &[]);
    let harness = Harness::start();
    harness.install_mod("echo", "echo", &plain, "echo.dll", 2);

    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    let reports = ModLoader::new(harness.state).load_all(&scan.mods);

    assert!(scan.skipped.is_empty(), "no minimum / plain / scan: {:?}", scan.skipped);
    assert_eq!(scan.mods[0].amml_version, None, "no minimum / plain / scan");
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].phase, ModPhase::Loaded, "no minimum / plain / load");
    assert!(reports[0].is_ok());
}

#[test]
fn folder_at_loader_version_loads() {
    let plain = EchoMod::build("plain", &[]);
    let harness = Harness::start();
    harness.install_mod_needing("echo", "echo", &plain, "echo.dll", 2, env!("CARGO_PKG_VERSION"));

    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    let reports = ModLoader::new(harness.state).load_all(&scan.mods);

    assert!(scan.skipped.is_empty(), "loader version / plain / scan: {:?}", scan.skipped);
    assert_eq!(scan.mods[0].amml_version, Some(ModLoader::amml_version()), "loader version / plain / scan");
    assert_eq!(reports[0].phase, ModPhase::Loaded, "loader version / plain / load");
    assert!(reports[0].is_ok());
}

#[test]
fn folder_one_patch_above_is_refused_and_others_load() {
    let plain = EchoMod::build("plain", &[]);
    let harness = Harness::start();
    let loader_version = ModLoader::amml_version();
    let above = AmmlVersion { patch: loader_version.patch + 1, ..loader_version };
    harness.install_mod_needing("echo", "gated", &plain, "gated.dll", 2, &above.to_string());
    harness.install_mod("echo", "echo", &plain, "echo.dll", 2);

    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    let reports = ModLoader::new(harness.state).load_all(&scan.mods);

    assert!(scan.skipped.is_empty(), "above loader / gated / scan: {:?}", scan.skipped);
    assert_eq!(reports.len(), 2);
    assert_eq!(reports[1].id, "@echo/gated");
    assert_eq!(reports[1].phase, ModPhase::Refused, "above loader / gated / load");
    let error = reports[1].error.as_ref().unwrap();
    assert!(
        matches!(error, ModLoadError::AmmlTooOld { needs, loader } if *needs == above && *loader == loader_version)
    );
    assert_eq!(error.status(), AmmlStatus::VERSION_MISMATCH);
    assert!(
        error.to_string().contains(&format!("needs AMML {above} or newer and this loader is {loader_version}")),
        "above loader / gated / reason: {error}"
    );
    assert_eq!(reports[0].id, "@echo/echo");
    assert_eq!(reports[0].phase, ModPhase::Loaded, "above loader / plain / load");
    assert!(reports[0].is_ok());
}

#[test]
fn folder_with_unknown_attribute_loads() {
    let plain = EchoMod::build("plain", &[]);
    let harness = Harness::start();
    harness.install_mod_with("echo", "echo", &plain, "echo.dll", 2, "future_field = \"x\"\n");

    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    let reports = ModLoader::new(harness.state).load_all(&scan.mods);

    assert!(scan.skipped.is_empty(), "unknown attribute / echo / scan: {:?}", scan.skipped);
    assert_eq!(scan.mods[0].ignored, ["attribute `future_field`"], "unknown attribute / echo / scan");
    assert_eq!(reports[0].phase, ModPhase::Loaded, "unknown attribute / echo / load");
    assert!(reports[0].is_ok());
}

#[test]
fn folder_with_malformed_amml_version_is_skipped() {
    let plain = EchoMod::build("plain", &[]);
    let harness = Harness::start();
    harness.install_mod_needing("echo", "broken", &plain, "broken.dll", 2, "26.62");
    harness.install_mod("echo", "echo", &plain, "echo.dll", 2);

    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    let reports = ModLoader::new(harness.state).load_all(&scan.mods);

    assert_eq!(scan.skipped.len(), 1, "malformed / broken / scan");
    assert!(scan.skipped[0].dir.ends_with("broken"));
    assert!(
        scan.skipped[0].reason.contains("`amml_version` must look like 26.62.0"),
        "malformed / broken / reason: {}",
        scan.skipped[0].reason
    );
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].id, "@echo/echo");
    assert_eq!(reports[0].phase, ModPhase::Loaded, "malformed / plain / load");
}
