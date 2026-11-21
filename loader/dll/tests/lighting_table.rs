use amml::{BlockHandles, LightingAdapter, ModLoader, SessionAdapter};
use amml_abi::{
    AmmlApi, AmmlBlockHandle, AmmlLightFalloff, AmmlLightSource, AmmlLightingTable, AmmlServices, AmmlStatus, AmmlStr,
    AmmlVec3,
};
use amml_runtime::ModRegistry;

use crate::support::{CLIENT_A, EventsMod, Harness, LogFile, SessionGame};

const MODULE: &str = "@events/events";

// A harness with the session adapter over the stand-in game and one loaded mod, `MODULE`.
fn start() -> Harness {
    let harness = Harness::start_with_entries(&SessionGame::entries(&[]));
    SessionAdapter::install(harness.state);
    let plain = EventsMod::build("plain", &[]);
    harness.install_mod("events", "events", &plain, "events.dll", 3);
    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    assert!(ModLoader::new(harness.state).load_all(&scan.mods).iter().all(|report| report.is_ok()));
    harness
}

// The lighting table, through the services table as a mod reaches it.
fn table(harness: &Harness) -> &'static AmmlLightingTable {
    // SAFETY: the loader's tables live for the process.
    let services = unsafe { AmmlApi::services(harness.state.api()) }.unwrap();
    // SAFETY: as above.
    let table = unsafe { AmmlServices::lighting(services) }.unwrap();
    // SAFETY: as above.
    unsafe { &*table }
}

fn source(block: Option<AmmlBlockHandle>) -> AmmlLightSource {
    AmmlLightSource {
        size: AmmlLightSource::SIZE_V1,
        version: 1,
        position: AmmlVec3 { x: 0.5, y: 65.0, z: 0.5 },
        level: 14,
        falloff: AmmlLightFalloff::ROUND,
        has_block: u8::from(block.is_some()),
        _reserved: 0,
        block: block.unwrap_or(AmmlBlockHandle::NONE),
    }
}

fn set(table: &AmmlLightingTable, module: AmmlStr, source: &AmmlLightSource) -> AmmlStatus {
    // SAFETY: the table is live; the view and the source live for the call.
    unsafe { (table.set_light_source)(module, source) }
}

#[test]
fn a_null_module_is_an_invalid_argument() {
    let harness = start();
    let module = AmmlStr { ptr: std::ptr::null(), len: 4 };

    assert_eq!(set(table(&harness), module, &source(None)), AmmlStatus::INVALID_ARGUMENT);
}

#[test]
fn without_the_lighting_adapter_the_area_is_not_available() {
    let harness = start();
    SessionGame::frame();
    let table = table(&harness);

    assert_eq!(set(table, AmmlStr::from_str(MODULE), &source(None)), AmmlStatus::NOT_AVAILABLE);
    harness.state.areas().offer(LightingAdapter::TERRAIN, 1);
    assert_eq!(set(table, AmmlStr::from_str(MODULE), &source(None)), AmmlStatus::NOT_AVAILABLE);
}

#[test]
fn before_the_first_frame_the_call_is_not_ready() {
    let harness = start();
    harness.state.areas().offer(LightingAdapter::TERRAIN, 1);

    assert_eq!(set(table(&harness), AmmlStr::from_str(MODULE), &source(None)), AmmlStatus::NOT_READY);
}

#[test]
fn off_the_game_thread_the_call_is_refused() {
    let harness = start();
    harness.state.areas().offer(LightingAdapter::TERRAIN, 1);
    SessionGame::frame();
    let table = table(&harness);

    let status = std::thread::spawn(move || set(table, AmmlStr::from_str(MODULE), &source(None))).join().unwrap();
    assert_eq!(status, AmmlStatus::REFUSED);
}

#[test]
fn a_block_handle_from_a_left_world_is_stale() {
    let harness = start();
    // No lighting entry resolves here, so the adapter hooks nothing; the area is offered by hand to reach its
    // handle check.
    LightingAdapter::install(harness.state);
    harness.state.areas().offer(LightingAdapter::TERRAIN, 1);
    SessionGame::frame();
    SessionGame::set_player(0x5000);
    SessionGame::start_join(&CLIENT_A, [0; 7]);
    SessionGame::update(&CLIENT_A);
    let epoch = harness.state.events().gate().epoch();
    assert_ne!(epoch.0, 0);
    let id = harness.state.resources().insert(BlockHandles::BLOCK, 0, epoch, 0x7000).unwrap();
    let handle = AmmlBlockHandle(id.0);

    SessionGame::exit();
    SessionGame::frame();
    SessionGame::frame();

    assert_eq!(set(table(&harness), AmmlStr::from_str(MODULE), &source(Some(handle))), AmmlStatus::STALE);
}

#[test]
fn disabling_a_mod_that_holds_a_light_clears_it() {
    let harness = Harness::start_with_entries(&SessionGame::entries(&[]));
    SessionAdapter::install(harness.state);
    let plain = EventsMod::build("plain", &[]);
    harness.install_mod("events", "events", &plain, "events.dll", 3);
    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    let loader = ModLoader::new(harness.state);
    assert!(loader.load_all(&scan.mods).iter().all(|report| report.is_ok()));
    loader.enable_all();
    // No lighting entry resolves here, so the adapter hooks nothing; the area is offered by hand.
    LightingAdapter::install(harness.state);
    harness.state.areas().offer(LightingAdapter::TERRAIN, 1);
    SessionGame::frame();
    assert_eq!(set(table(&harness), AmmlStr::from_str(MODULE), &source(None)), AmmlStatus::OK);

    loader.disable_one(0);

    let line = format!("lighting cleared {{\"mod_name\":\"{MODULE}\"}}");
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), &line);
    assert!(log.contains(&line), "{log}");
}

#[test]
fn leaving_the_world_puts_the_lights_out_before_a_disable() {
    let harness = Harness::start_with_entries(&SessionGame::entries(&[]));
    SessionAdapter::install(harness.state);
    let plain = EventsMod::build("plain", &[]);
    harness.install_mod("events", "events", &plain, "events.dll", 3);
    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    let loader = ModLoader::new(harness.state);
    assert!(loader.load_all(&scan.mods).iter().all(|report| report.is_ok()));
    loader.enable_all();
    // No lighting entry resolves here, so the adapter hooks nothing; the area is offered by hand.
    LightingAdapter::install(harness.state);
    harness.state.areas().offer(LightingAdapter::TERRAIN, 1);
    SessionGame::frame();
    SessionGame::set_player(0x5000);
    SessionGame::start_join(&CLIENT_A, [0; 7]);
    SessionGame::update(&CLIENT_A);
    assert_eq!(set(table(&harness), AmmlStr::from_str(MODULE), &source(None)), AmmlStatus::OK);

    SessionGame::exit();
    loader.disable_one(0);

    // The world leave took the light, so the disable finds none to clear: the shutdown order of the game.
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "mod disable");
    assert!(log.contains("mod disable"), "{log}");
    assert!(!log.contains("lighting cleared"), "{log}");
}
