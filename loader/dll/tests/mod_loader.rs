use std::ffi::c_void;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use amml::{ClientAdapter, EventHub, LoaderState, ModListener, ModLoadError, ModLoader, ModPhase, SessionAdapter};
use amml_abi::{
    AmmlApi, AmmlBand, AmmlClientFlag, AmmlEventContext, AmmlEventHeader, AmmlPhase, AmmlServices, AmmlStatus, AmmlStr,
    ClientTick, FrameUpdate, WorldLeft,
};
use amml_runtime::{AmmlVersion, ModRegistry, ModSpec};

use crate::support::{
    CLIENT_A, ClientGame, EchoMod, EventsMod, GAME_CLIENT_A, Harness, HeapProbe, HookTarget, LogFile, SessionGame,
};

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

    // The manifest says 2 but the DLL needs loader API 4, so its own load call refuses this loader.
    let mut reports = loader.load_all(&scan.mods);
    // A manifest asking for version 4 is refused before the DLL is mapped.
    let ahead = ModSpec {
        namespace: "echo".to_string(),
        name: "ahead".to_string(),
        dir: harness.mods().join("@echo").join("ahead"),
        entry: "missing.dll".to_string(),
        api_version: 4,
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

// Installs each `(name, dll)` as `@events/<name>` and loads every mod.
fn load_events_mods(harness: &Harness, mods: &[(&str, &std::path::Path)]) -> ModLoader {
    for (name, dll) in mods {
        harness.install_mod("events", name, dll, &format!("{}.dll", name.replace('-', "_")), 3);
    }
    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    let loader = ModLoader::new(harness.state);
    assert!(loader.load_all(&scan.mods).iter().all(|report| report.is_ok()));
    loader
}

fn tick() -> ClientTick {
    ClientTick { header: EventHub::header::<ClientTick>(AmmlPhase::AFTER) }
}

#[test]
fn a_v2_mod_still_loads_and_hooks() {
    let dll = EchoMod::build("hook", &["hook-target"]);
    let harness = Harness::start();
    harness.install_mod("echo", "echo-hook", &dll, "echo_hook.dll", 2);
    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    let loader = ModLoader::new(harness.state);

    assert!(loader.load_all(&scan.mods).iter().all(|report| report.is_ok()));
    assert!(loader.enable_all().iter().all(|report| report.is_ok()));

    // SAFETY: the loader's table lives for the process.
    let header = unsafe { AmmlApi::header(harness.state.api()) };
    assert_eq!(header, Some((AmmlApi::SIZE_V3, 3)));
    // SAFETY: as above.
    assert!(unsafe { AmmlApi::services(harness.state.api()) }.is_some());
    assert_eq!(loader.phases(), [("@echo/echo-hook".to_string(), ModPhase::Enabled)]);
    assert_eq!(HookTarget::call(1), 1002);
}

#[test]
fn disable_drops_providers_then_removes_listeners_then_disables() {
    let plain = EventsMod::build("plain", &[]);
    let provider = EventsMod::build("provider", &["provider"]);
    let harness = Harness::start();
    let hub = harness.state.events();
    hub.offer("client.tick", 1);
    let loader = load_events_mods(&harness, &[("events", &plain), ("events-provider", &provider)]);
    loader.enable_all();
    let mods = harness.mods();
    assert_eq!(EventsMod::lines_of(&mods, "events-provider", "provide"), ["events-provider provide status=0"]);
    let ready = EventsMod::lines_of(&mods, "events", "registry.provider_ready");
    assert_eq!(ready, ["events registry.provider_ready thread=0 epoch=0 alive=0 name=test.table major=1 minor=0"]);

    loader.disable_one(1);
    hub.raise(&mut tick(), &hub.loader_context());

    assert_eq!(EventsMod::lines_of(&mods, "events-provider", "client.tick").len(), 0);
    assert_eq!(EventsMod::lines_of(&mods, "events", "client.tick").len(), 1);
    assert_eq!(harness.state.require("test.table", 1, 0), Err(AmmlStatus::NOT_AVAILABLE));
    let gone = EventsMod::lines_of(&mods, "events", "registry.provider_gone");
    assert_eq!(gone, ["events registry.provider_gone thread=0 epoch=0 alive=0 name=test.table major=1"]);
    assert!(EventsMod::lines_of(&mods, "events-provider", "registry.provider_gone").is_empty());
    // Consumers hear of the dropped table before the provider's own disable runs.
    let lines = EventsMod::lines(&mods);
    let position = |needle: &str| lines.iter().position(|line| line.starts_with(needle)).unwrap();
    assert!(position("events registry.provider_gone") < position("events-provider disable listen"), "{lines:?}");
    assert_eq!(loader.phases()[1], ("@events/events-provider".to_string(), ModPhase::Disabled));
}

static NESTED_DISABLES: AtomicUsize = AtomicUsize::new(0);

// Tries to disable mod 1 again while its disable raises `ProviderGone`.
unsafe extern "C" fn disable_again(
    _event: *mut AmmlEventHeader,
    _context: *const AmmlEventContext,
    _user: *mut c_void,
) -> AmmlStatus {
    if let Some(state) = LoaderState::get() {
        ModLoader::new(state).disable_one(1);
        NESTED_DISABLES.fetch_add(1, Ordering::SeqCst);
    }
    AmmlStatus::OK
}

#[test]
fn a_second_disable_during_the_first_does_nothing() {
    let plain = EventsMod::build("plain", &[]);
    let provider = EventsMod::build("provider", &["provider"]);
    let harness = Harness::start();
    let loader = load_events_mods(&harness, &[("events", &plain), ("events-provider", &provider)]);
    loader.enable_all();
    let listener = ModListener { owner: 50, callback: disable_again, user: 0 };
    harness.state.events().listen("registry.provider_gone", 1, AmmlBand::NORMAL, listener).unwrap();

    loader.disable_one(1);

    assert_eq!(NESTED_DISABLES.load(Ordering::SeqCst), 1);
    assert_eq!(EventsMod::lines_of(&harness.mods(), "events-provider", "disable listen").len(), 1);
    assert_eq!(loader.phases()[1], ("@events/events-provider".to_string(), ModPhase::Disabled));
}

#[test]
fn a_missing_required_event_fails_load() {
    let required = EventsMod::build("load-required", &["require-in-load"]);
    let harness = Harness::start();
    harness.install_mod("events", "events-load-required", &required, "events_load_required.dll", 3);
    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    let loader = ModLoader::new(harness.state);

    let reports = loader.load_all(&scan.mods);

    assert_eq!(reports[0].phase, ModPhase::Failed);
    assert!(matches!(&reports[0].error, Some(ModLoadError::RequiredEvent { feature, event })
            if feature == "ticker" && event == "client.tick"));
    assert!(loader.enable_all().is_empty());
}

#[test]
fn shutdown_comes_before_every_disable_and_keeps_the_gate_shut() {
    let plain = EventsMod::build("plain", &[]);
    let harness = Harness::start();
    let loader = load_events_mods(&harness, &[("events", &plain)]);
    loader.enable_all();
    let hub = harness.state.events();
    let mut frame = FrameUpdate { header: EventHub::header::<FrameUpdate>(AmmlPhase::BEFORE) };
    hub.raise(&mut frame, &hub.loader_context());
    assert!(hub.gate().is_open());

    loader.shutdown();

    let lines = EventsMod::lines(&harness.mods());
    let position = |needle: &str| lines.iter().position(|line| line == needle).unwrap();
    let shutdown = position("events loader.shutdown thread=0 epoch=0 alive=0");
    let disable = position(&format!("events disable listen status={} game_alive=0", AmmlStatus::FAILED.0));
    assert!(shutdown < disable, "{lines:?}");
    assert_eq!(loader.phases(), [("@events/events".to_string(), ModPhase::Disabled)]);
    let mut frame = FrameUpdate { header: EventHub::header::<FrameUpdate>(AmmlPhase::BEFORE) };
    hub.raise(&mut frame, &hub.loader_context());
    assert!(!hub.gate().is_open());
}

#[test]
fn a_missing_required_event_fails_enable() {
    let required = EventsMod::build("required", &["require-tick"]);
    let harness = Harness::start();
    let loader = load_events_mods(&harness, &[("events-required", &required)]);

    let reports = loader.enable_all();

    assert_eq!(reports[0].phase, ModPhase::Failed);
    assert_eq!(reports[0].error.as_ref().map(ModLoadError::status), Some(AmmlStatus::NOT_AVAILABLE));
    assert_eq!(loader.phases(), [("@events/events-required".to_string(), ModPhase::Failed)]);
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "mod switched off");
    assert!(log.contains("ticker") && log.contains("client.tick"), "{log}");
}

#[test]
fn a_required_event_of_the_uses_block_is_checked_before_enable_runs() {
    let required = EventsMod::build("required", &["require-tick"]);
    let harness = Harness::start();
    let loader = load_events_mods(&harness, &[("events-required", &required)]);

    let reports = loader.enable_all();

    assert_eq!(reports[0].phase, ModPhase::Failed);
    assert!(matches!(&reports[0].error, Some(ModLoadError::RequiredEvent { feature, event })
            if feature == "ticker" && event == "client.tick"));
    assert!(EventsMod::lines_of(&harness.mods(), "events-required", "enable").is_empty());
}

#[test]
fn an_optional_listen_of_an_event_not_offered_logs_the_feature_that_is_off() {
    let plain = EventsMod::build("plain", &[]);
    let harness = Harness::start();
    let loader = load_events_mods(&harness, &[("events", &plain)]);

    let reports = loader.enable_all();

    assert_eq!(reports[0].phase, ModPhase::Enabled);
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "feature off");
    let line = log.lines().find(|line| line.contains("feature off")).unwrap();
    assert!(line.contains("@events/events") && line.contains("ticker") && line.contains("client.tick"), "{log}");
}

#[test]
fn a_missing_required_area_fails_enable() {
    let area = EventsMod::build("area", &["require-area"]);
    let harness = Harness::start();
    let loader = load_events_mods(&harness, &[("events-area", &area)]);

    let reports = loader.enable_all();

    assert_eq!(reports[0].phase, ModPhase::Failed);
    assert!(matches!(&reports[0].error, Some(ModLoadError::RequiredArea { feature, area })
            if feature == "keys" && area == "input.key_inject"));
    assert_eq!(reports[0].error.as_ref().map(ModLoadError::status), Some(AmmlStatus::NOT_AVAILABLE));
    assert!(EventsMod::lines_of(&harness.mods(), "events-area", "enable").is_empty());
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "a required area is not available");
    let line = log.lines().find(|line| line.contains("a required area is not available")).unwrap();
    assert!(
        line.contains("@events/events-area") && line.contains("keys") && line.contains("input.key_inject"),
        "{log}"
    );
}

#[test]
fn a_missing_optional_area_logs_the_feature_that_is_off_and_enables() {
    let area = EventsMod::build("optional-area", &["optional-area"]);
    let harness = Harness::start();
    let loader = load_events_mods(&harness, &[("events-optional-area", &area)]);

    let reports = loader.enable_all();

    assert_eq!(reports[0].phase, ModPhase::Enabled);
    assert!(reports[0].error.is_none());
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "feature off: its area is not available");
    let line = log.lines().find(|line| line.contains("feature off: its area is not available")).unwrap();
    assert!(
        line.contains("@events/events-optional-area") && line.contains("keys") && line.contains("input.key_inject"),
        "{log}"
    );
    assert!(!log.contains("a required area is not available"), "{log}");
    let too_old = log.lines().find(|line| line.contains("loader too old")).unwrap_or_default();
    assert!(
        too_old.contains("future") && too_old.contains("actor.read") && too_old.contains("\"served\":\"4\""),
        "{log}"
    );
    assert_eq!(log.matches("loader too old").count(), 1, "{log}");
}

#[test]
fn the_core_available_call_answers_areas() {
    let harness = Harness::start();
    harness.state.areas().offer("input.lease", 1);
    // SAFETY: the loader's table lives for the process.
    let services = unsafe { AmmlApi::services(harness.state.api()) }.unwrap();
    // SAFETY: the loader keeps its service tables alive for the process.
    let core = unsafe { &*(*services).core };
    // SAFETY: each view lives for its call.
    let available = |area: &str, version: u32| unsafe { (core.available)(AmmlStr::from_str(area), version) };

    assert_eq!(available("input.lease", 1), AmmlStatus::OK);
    assert_eq!(available("input.lease", 2), AmmlStatus::NOT_AVAILABLE);
    assert_eq!(available("input.key_inject", 1), AmmlStatus::NOT_AVAILABLE);
}

#[test]
fn mods_loaded_comes_once_after_every_enable() {
    let plain = EventsMod::build("plain", &[]);
    let provider = EventsMod::build("provider", &["provider"]);
    let harness = Harness::start();
    let loader = load_events_mods(&harness, &[("events", &plain), ("events-provider", &provider)]);

    loader.enable_all();
    loader.enable_all();

    let lines = EventsMod::lines(&harness.mods());
    let position = |needle: &str| lines.iter().position(|line| line == needle).unwrap();
    let last_enable = position("events enable").max(position("events-provider enable"));
    for name in ["events", "events-provider"] {
        let loaded = EventsMod::lines_of(&harness.mods(), name, "mods.loaded");
        assert_eq!(loaded.len(), 1, "{name}: {lines:?}");
        assert!(position(&loaded[0]) > last_enable, "{lines:?}");
    }
}

#[test]
fn the_game_gate_stays_closed_until_the_first_frame() {
    let plain = EventsMod::build("plain", &[]);
    let harness = Harness::start();
    let loader = load_events_mods(&harness, &[("events", &plain)]);
    loader.enable_all();
    let hub = harness.state.events();

    assert!(!hub.gate().is_open());
    assert_eq!(hub.gate().require_open(), Err(AmmlStatus::NOT_READY));

    let mut frame = FrameUpdate { header: EventHub::header::<FrameUpdate>(AmmlPhase::BEFORE) };
    hub.raise(&mut frame, &hub.loader_context());

    assert!(hub.gate().is_open());
    assert_eq!(hub.gate().require_open(), Ok(()));
}

static PROVIDER_GONE_SEEN: AtomicUsize = AtomicUsize::new(0);

unsafe extern "C" fn count_provider_gone(
    _event: *mut AmmlEventHeader,
    _context: *const AmmlEventContext,
    _user: *mut c_void,
) -> AmmlStatus {
    PROVIDER_GONE_SEEN.fetch_add(1, Ordering::SeqCst);
    AmmlStatus::OK
}

#[test]
fn a_provider_that_fails_its_enable_raises_provider_gone_to_a_consumer_that_required_it() {
    let failing = EventsMod::build("provider-fails", &["provider-fails"]);
    let harness = Harness::start();
    let loader = load_events_mods(&harness, &[("events-provider-fails", &failing)]);
    // A consumer took the table between the provider's load and its enable.
    assert!(harness.state.require("test.table", 1, 0).is_ok());
    let consumer = ModListener { owner: 50, callback: count_provider_gone, user: 0 };
    harness.state.events().listen("registry.provider_gone", 1, AmmlBand::NORMAL, consumer).unwrap();

    let reports = loader.enable_all();

    assert_eq!(reports[0].phase, ModPhase::Failed);
    assert_eq!(PROVIDER_GONE_SEEN.load(Ordering::SeqCst), 1);
    assert_eq!(harness.state.require("test.table", 1, 0), Err(AmmlStatus::NOT_AVAILABLE));
}

static EVENTS_LOG: OnceLock<PathBuf> = OnceLock::new();
static BLOCKED_STARTED: AtomicBool = AtomicBool::new(false);

// A listener of mod 1 that is still running when the disable starts: it ends 300 ms later and
// writes its end to the events log.
unsafe extern "C" fn slow_listener(
    _event: *mut AmmlEventHeader,
    _context: *const AmmlEventContext,
    _user: *mut c_void,
) -> AmmlStatus {
    BLOCKED_STARTED.store(true, Ordering::SeqCst);
    std::thread::sleep(Duration::from_millis(300));
    if let Some(path) = EVENTS_LOG.get()
        && let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(path)
    {
        let _ = writeln!(file, "slow listener ended");
    }
    AmmlStatus::OK
}

#[test]
fn a_listener_running_on_another_thread_ends_before_the_mod_disable_runs() {
    let plain = EventsMod::build("plain", &[]);
    let provider = EventsMod::build("provider", &["provider"]);
    let harness = Harness::start();
    let hub = harness.state.events();
    hub.offer("client.tick", 1);
    let loader = load_events_mods(&harness, &[("events", &plain), ("events-provider", &provider)]);
    loader.enable_all();
    EVENTS_LOG.set(harness.mods().join("events.log")).unwrap();
    let slow = ModListener { owner: 1, callback: slow_listener, user: 0 };
    hub.listen("client.tick", 1, AmmlBand::LAST, slow).unwrap();

    let raiser = std::thread::spawn(move || {
        hub.raise(&mut tick(), &hub.loader_context());
    });
    let deadline = Instant::now() + Duration::from_secs(10);
    while !BLOCKED_STARTED.load(Ordering::SeqCst) {
        assert!(Instant::now() < deadline, "the slow listener never started");
        std::thread::yield_now();
    }
    loader.disable_one(1);
    raiser.join().unwrap();

    let lines = EventsMod::lines(&harness.mods());
    let position = |needle: &str| lines.iter().position(|line| line.starts_with(needle)).unwrap();
    assert!(position("slow listener ended") < position("events-provider disable listen"), "{lines:?}");
    assert_eq!(loader.phases()[1], ("@events/events-provider".to_string(), ModPhase::Disabled));
}

// One line per `world.left` or `loader.shutdown` a test listener saw.
static CLOSING_SEEN: Mutex<Vec<String>> = Mutex::new(Vec::new());

// The events of `record_closing`; a listener's `user` value is its index here.
const CLOSING_EVENTS: [&str; 2] = ["world.left", "loader.shutdown"];

// Records `<event> alive=<game_alive> epoch=<context epoch> client=<context client>`, and the world of a `WorldLeft`.
unsafe extern "C" fn record_closing(
    event: *mut AmmlEventHeader,
    context: *const AmmlEventContext,
    user: *mut c_void,
) -> AmmlStatus {
    let id = CLOSING_EVENTS[user as usize];
    // SAFETY: the loader passes a live context for the call.
    let context = unsafe { &*context };
    let world = if id == "world.left" {
        // SAFETY: the loader raises `world.left` with a `WorldLeft`.
        format!(" world={}", unsafe { (*event.cast::<WorldLeft>()).epoch })
    } else {
        String::new()
    };
    CLOSING_SEEN.lock().unwrap().push(format!(
        "{id} alive={} epoch={} client={:#x}{world}",
        context.game_alive, context.epoch, context.client as usize
    ));
    AmmlStatus::OK
}

fn listen_closing(harness: &Harness) {
    for (index, event) in CLOSING_EVENTS.iter().enumerate() {
        let listener = ModListener { owner: 40, callback: record_closing, user: index };
        harness.state.events().listen(event, 1, AmmlBand::NORMAL, listener).unwrap();
    }
}

#[test]
fn shutdown_live_leaves_the_joined_world_before_shutdown() {
    let harness = Harness::start_with_entries(&SessionGame::entries(&[]));
    SessionAdapter::install(harness.state);
    listen_closing(&harness);
    SessionGame::frame();
    SessionGame::set_player(0x5000);
    SessionGame::start_join(&CLIENT_A, [0; 7]);
    SessionGame::update(&CLIENT_A);
    assert_eq!(harness.state.events().gate().epoch().0, 1);
    let client = SessionGame::client(&CLIENT_A) as usize;

    ModLoader::new(harness.state).shutdown_live(client);

    assert_eq!(
        *CLOSING_SEEN.lock().unwrap(),
        [
            format!("world.left alive=1 epoch=1 client={client:#x} world=1"),
            format!("loader.shutdown alive=1 epoch=0 client={client:#x}"),
        ]
    );
    assert_eq!(harness.state.events().gate().epoch().0, 0);
    assert!(!harness.state.events().gate().is_open());
    assert_eq!(SessionAdapter::primary_client(), None);
}

#[test]
fn hud_is_restored_before_the_gate_closes() {
    let harness = Harness::start_with_entries(&ClientGame::entries(&[]));
    SessionAdapter::install(harness.state);
    ClientAdapter::install(harness.state);
    ClientGame::frame();
    ClientGame::update(&GAME_CLIENT_A);
    ClientGame::set_hide_hud(false);
    let plain = EventsMod::build("plain", &[]);
    let loader = load_events_mods(&harness, &[("events", &plain)]);
    loader.enable_all();
    // SAFETY: the loader's tables live for the process.
    let services = unsafe { AmmlApi::services(harness.state.api()) }.unwrap();
    // SAFETY: as above.
    let table = unsafe { &*AmmlServices::client(services).unwrap() };
    let client = ClientGame::client(&GAME_CLIENT_A);
    // SAFETY: the table is live and the view lives for the call.
    let status = unsafe {
        (table.set_flag)(AmmlStr::from_str("@events/events"), client.cast_const(), AmmlClientFlag::HIDE_HUD, 1)
    };
    assert_eq!(status, AmmlStatus::OK);

    loader.shutdown_live(client as usize);

    assert_eq!(ClientGame::hud_sets(), [true, false]);
    assert_eq!(loader.phases(), [("@events/events".to_string(), ModPhase::Disabled)]);
    assert!(!harness.state.events().gate().is_open());
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "client flag restored");
    assert!(log.contains("HideHud"), "{log}");
}

#[test]
fn a_shutdown_after_the_live_shutdown_does_nothing() {
    let plain = EventsMod::build("plain", &[]);
    let harness = Harness::start();
    let loader = load_events_mods(&harness, &[("events", &plain)]);
    loader.enable_all();
    let hub = harness.state.events();
    hub.raise(&mut FrameUpdate { header: EventHub::header::<FrameUpdate>(AmmlPhase::BEFORE) }, &hub.loader_context());
    hub.gate().note_game_thread();

    loader.shutdown_live(0);
    loader.shutdown_live(0);
    loader.shutdown();

    let mods = harness.mods();
    assert_eq!(
        EventsMod::lines_of(&mods, "events", "loader.shutdown"),
        ["events loader.shutdown thread=0 epoch=0 alive=1"]
    );
    let disables = EventsMod::lines_of(&mods, "events", "disable listen");
    assert_eq!(disables, [format!("events disable listen status={} game_alive=1", AmmlStatus::FAILED.0)]);
    assert!(!hub.gate().is_open());
    assert!(harness.state.shutdown().is_closing());
}
