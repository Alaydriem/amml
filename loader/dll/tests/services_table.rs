use std::ffi::c_void;

use amml::{EventHub, ModListener, ModLoader, ModPhase};
use amml_abi::{
    AmmlApi, AmmlBand, AmmlDirKind, AmmlEventContext, AmmlEventHeader, AmmlLogLevel, AmmlPhase, AmmlServices,
    AmmlStatus, AmmlStr, AmmlThread, ClientTick, FrameUpdate,
};
use amml_runtime::ModRegistry;

use crate::support::{EventsMod, Harness, LogFile};

fn services(harness: &Harness) -> &'static AmmlServices {
    // SAFETY: the loader's table lives for the process.
    let services = unsafe { AmmlApi::services(harness.state.api()) }.unwrap();
    // SAFETY: the loader keeps its service tables alive for the process.
    unsafe { &*services }
}

fn context() -> AmmlEventContext {
    AmmlEventContext {
        size: 32,
        version: 1,
        thread: AmmlThread::GAME,
        game_alive: 0,
        _reserved: [0; 6],
        epoch: 0,
        client: std::ptr::null(),
    }
}

fn tick() -> ClientTick {
    ClientTick { header: EventHub::header::<ClientTick>(AmmlPhase::AFTER) }
}

unsafe extern "C" fn never_called(
    _event: *mut AmmlEventHeader,
    _context: *const AmmlEventContext,
    _user: *mut c_void,
) -> AmmlStatus {
    AmmlStatus::OK
}

// Installs each `(name, dll)` as `@events/<name>`, then loads and enables every mod.
fn start_mods(harness: &Harness, mods: &[(&str, &std::path::Path)]) -> ModLoader {
    for (name, dll) in mods {
        harness.install_mod("events", name, dll, &format!("{}.dll", name.replace('-', "_")), 3);
    }
    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    let loader = ModLoader::new(harness.state);
    assert!(loader.load_all(&scan.mods).iter().all(|report| report.is_ok()));
    loader
}

#[test]
fn a_listen_outside_load_and_enable_is_refused() {
    let harness = Harness::start();
    harness.state.events().offer("client.tick", 1);
    let events = services(&harness).events;
    let mut handle = 0u64;

    // SAFETY: the table is live; the views and the handle slot live for the call.
    let status = unsafe {
        ((*events).listen)(
            AmmlStr::from_str("client.tick"),
            1,
            AmmlBand::NORMAL,
            0,
            Some(never_called),
            std::ptr::null_mut(),
            &mut handle,
        )
    };

    assert_eq!(status, AmmlStatus::FAILED);
}

#[test]
fn a_listen_with_a_null_callback_is_an_invalid_argument() {
    let harness = Harness::start();
    let events = services(&harness).events;
    let mut handle = 0u64;

    // SAFETY: the table is live; the views and the handle slot live for the call.
    let status = unsafe {
        ((*events).listen)(
            AmmlStr::from_str("mods.loaded"),
            1,
            AmmlBand::NORMAL,
            0,
            None,
            std::ptr::null_mut(),
            &mut handle,
        )
    };

    assert_eq!(status, AmmlStatus::INVALID_ARGUMENT);
}

#[test]
fn unlisten_removes_a_listener_once() {
    let harness = Harness::start();
    let hub = harness.state.events();
    let listener = ModListener { owner: 7, callback: never_called, user: 0 };
    let id = hub.listen("mods.loaded", 1, AmmlBand::NORMAL, listener).unwrap();
    let events = services(&harness).events;

    // SAFETY: the table is live.
    let (first, second) = unsafe { (((*events).unlisten)(id.0), ((*events).unlisten)(id.0)) };

    assert_eq!((first, second), (AmmlStatus::OK, AmmlStatus::INVALID_ARGUMENT));
}

#[test]
fn game_alive_follows_the_game_gate_and_stays_off_after_close() {
    let harness = Harness::start();
    let hub = harness.state.events();
    // SAFETY: the loader keeps its service tables alive for the process.
    let core = unsafe { &*services(&harness).core };
    // SAFETY: the table is live.
    let alive = || unsafe { (core.game_alive)() };
    let frame = || {
        let mut frame = FrameUpdate { header: EventHub::header::<FrameUpdate>(AmmlPhase::BEFORE) };
        hub.raise(&mut frame, &context());
    };

    assert_eq!(alive(), 0);
    frame();
    assert_eq!(alive(), 1);
    hub.gate().close();
    assert_eq!(alive(), 0);
    frame();
    assert_eq!(alive(), 0);
    assert!(!hub.gate().is_open());
}

#[test]
fn a_listen_on_an_unoffered_event_is_not_available() {
    let dll = EventsMod::build("plain", &[]);
    let harness = Harness::start();
    let loader = start_mods(&harness, &[("events", &dll)]);

    loader.enable_all();

    let lines = EventsMod::lines_of(&harness.mods(), "events", "listen client.tick");
    assert_eq!(lines, [format!("events listen client.tick status={}", AmmlStatus::NOT_AVAILABLE.0)]);
    assert_eq!(loader.phases(), [("@events/events".to_string(), ModPhase::Enabled)]);
}

#[test]
fn a_raised_event_reaches_the_listener_with_its_context() {
    let dll = EventsMod::build("plain", &[]);
    let harness = Harness::start();
    let hub = harness.state.events();
    hub.offer("client.tick", 1);
    let loader = start_mods(&harness, &[("events", &dll)]);
    loader.enable_all();

    let report = hub.raise(&mut tick(), &context());

    assert!(report.panicked_owners.is_empty() && !report.cancelled);
    let lines = EventsMod::lines_of(&harness.mods(), "events", "client.tick");
    assert_eq!(lines, ["events client.tick thread=0 epoch=0 alive=0"]);
}

#[test]
fn a_panicking_listener_disables_its_mod_on_the_next_frame() {
    let plain = EventsMod::build("plain", &[]);
    let panicking = EventsMod::build("panic", &["panic-listener"]);
    let harness = Harness::start();
    let hub = harness.state.events();
    hub.offer("client.tick", 1);
    let loader = start_mods(&harness, &[("events", &plain), ("events-panic", &panicking)]);
    loader.enable_all();

    let report = hub.raise(&mut tick(), &context());
    assert_eq!(report.panicked_owners, [1]);
    // The mod stays enabled until the next frame drains the deferred disables.
    assert_eq!(loader.phases()[1], ("@events/events-panic".to_string(), ModPhase::Enabled));
    hub.deferred().drain(&loader);
    hub.raise(&mut tick(), &context());

    assert_eq!(
        loader.phases(),
        [("@events/events".to_string(), ModPhase::Enabled), ("@events/events-panic".to_string(), ModPhase::Disabled)]
    );
    assert_eq!(EventsMod::lines_of(&harness.mods(), "events", "client.tick").len(), 2);
    assert_eq!(EventsMod::lines_of(&harness.mods(), "events-panic", "client.tick").len(), 1);
    let disable = EventsMod::lines_of(&harness.mods(), "events-panic", "disable listen");
    assert_eq!(disable, [format!("events-panic disable listen status={} game_alive=0", AmmlStatus::FAILED.0)]);
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "after a listener panic");
    assert!(log.contains("mod disabled after a listener panic") && log.contains("@events/events-panic"), "{log}");
}

#[test]
fn core_and_registry_serve_folders_fields_availability_and_tables() {
    let plain = EventsMod::build("plain", &[]);
    let provider = EventsMod::build("provider", &["provider"]);
    let harness = Harness::start();
    let loader = start_mods(&harness, &[("events", &plain), ("events-provider", &provider)]);
    loader.enable_all();
    let services = services(&harness);
    // SAFETY: the loader keeps its service tables alive for the process.
    let (core, registry) = unsafe { (&*services.core, &*services.registry) };
    let module = AmmlStr::from_str("@events/events");
    let dir = |kind: AmmlDirKind| {
        let mut out = AmmlStr::EMPTY;
        // SAFETY: the views live for the call and `out` is writable.
        let status = unsafe { (core.dir)(module, kind, &mut out) };
        // SAFETY: the loader keeps the folder bytes valid for the process.
        (status, unsafe { out.as_str() }.unwrap().to_string())
    };
    // SAFETY: each view lives for its call.
    let available = |event: &str| unsafe { (core.available)(AmmlStr::from_str(event), 1) };

    let mod_dir = harness.mods().join("@events").join("events");
    assert_eq!(dir(AmmlDirKind::MOD), (AmmlStatus::OK, mod_dir.display().to_string()));
    assert_eq!(dir(AmmlDirKind::CONFIG), (AmmlStatus::OK, mod_dir.join("config").display().to_string()));
    // The harness gives the loader no instance folder.
    assert_eq!(dir(AmmlDirKind::INSTANCE).0, AmmlStatus::NOT_AVAILABLE);
    assert_eq!(dir(AmmlDirKind(9)).0, AmmlStatus::INVALID_ARGUMENT);
    assert_eq!(available("mods.loaded"), AmmlStatus::OK);
    assert_eq!(available("client.tick"), AmmlStatus::NOT_AVAILABLE);

    let mut table: *const c_void = std::ptr::null();
    let mut minor = 9u16;
    // SAFETY: the view lives for the call and both outputs are writable.
    let status = unsafe { (registry.require)(AmmlStr::from_str("test.table"), 1, 0, &mut table, &mut minor) };
    assert_eq!((status, table.is_null(), minor), (AmmlStatus::OK, false, 0));
    // SAFETY: as above.
    let status = unsafe { (registry.require)(AmmlStr::from_str("test.table"), 2, 0, &mut table, &mut minor) };
    assert_eq!(status, AmmlStatus::NOT_AVAILABLE);
    // A provide from a plain thread is refused: no mod callback runs.
    // SAFETY: the view lives for the call; the table pointer is never read.
    let status = unsafe { (registry.provide)(AmmlStr::from_str("other.table"), 1, 0, (&raw const minor).cast()) };
    assert_eq!(status, AmmlStatus::FAILED);

    let keys = [AmmlStr::from_str("a"), AmmlStr::from_str("b")];
    let values = [AmmlStr::from_str("1"), AmmlStr::from_str("two")];
    // SAFETY: the views live for the call and both arrays hold two views.
    unsafe {
        (core.log)(module, AmmlLogLevel::INFO, AmmlStr::from_str("fields line"), keys.as_ptr(), values.as_ptr(), 2);
    }
    let log = LogFile::wait_for(&harness.logs().join("@events").join("events.log"), "fields line");
    assert!(log.contains("fields line a=1 b=two"), "{log}");
}
