use std::ffi::c_void;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};

use amml::{ClientAdapter, LevelRendererAdapter, ModListener, SessionAdapter};
use amml_abi::{AmmlBand, AmmlEventContext, AmmlEventHeader, AmmlStatus, CameraFov, DimensionChanging};
use amml_data::Availability;

use crate::support::{ClientGame, GAME_CLIENT_A, Harness, LevelRendererGame, SessionGame};

// What the `camera.fov` listener saw, one line per event.
static SEEN: Mutex<Vec<String>> = Mutex::new(Vec::new());
// The `result` the fov listener writes, as `f32` bits; `NO_WRITE` writes `steady / 4`.
static WRITE: AtomicU32 = AtomicU32::new(NO_WRITE);
// When set, the fov listener returns `PANICKED` after its write.
static FAIL: AtomicU32 = AtomicU32::new(0);

const NO_WRITE: u32 = u32::MAX;

unsafe extern "C" fn on_fov(event: *mut AmmlEventHeader, _: *const AmmlEventContext, _: *mut c_void) -> AmmlStatus {
    // SAFETY: the loader raises `camera.fov` with a `CameraFov`.
    let fov = unsafe { &mut *event.cast::<CameraFov>() };
    SEEN.lock().unwrap().push(format!("fov base={} steady={} result={}", fov.base, fov.steady, fov.result));
    fov.result = match WRITE.load(Ordering::SeqCst) {
        NO_WRITE => fov.steady / 4.0,
        bits => f32::from_bits(bits),
    };
    if FAIL.load(Ordering::SeqCst) != 0 { AmmlStatus::PANICKED } else { AmmlStatus::OK }
}

unsafe extern "C" fn on_dimension(
    event: *mut AmmlEventHeader,
    _: *const AmmlEventContext,
    _: *mut c_void,
) -> AmmlStatus {
    // SAFETY: the loader raises `world.dimension_changing` with a `DimensionChanging`.
    let _ = unsafe { &*event.cast::<DimensionChanging>() };
    LevelRendererGame::record("dimension");
    AmmlStatus::OK
}

fn seen() -> Vec<String> {
    SEEN.lock().unwrap().clone()
}

fn dimension_events() -> usize {
    LevelRendererGame::calls().iter().filter(|line| *line == "dimension").count()
}

// Installs the session, client and level renderer adapters over the stand-ins without the entries in
// `without`; gives the harness and the level renderer rows.
fn install(without: &[&str]) -> (Harness, Vec<Availability>) {
    let harness = Harness::start_with_entries(&LevelRendererGame::entries(without));
    SessionAdapter::install(harness.state);
    ClientAdapter::install(harness.state);
    let rows = LevelRendererAdapter::install(harness.state);
    (harness, rows)
}

fn listen(harness: &Harness, id: &str, callback: amml_abi::EventCallbackFn) -> bool {
    let listener = ModListener { owner: 0, callback, user: 0 };
    harness.state.events().listen(id, 1, AmmlBand::NORMAL, listener).is_ok()
}

// `install` with every entry, a `camera.fov` listener and a `world.dimension_changing` listener.
fn start() -> Harness {
    let (harness, _) = install(&[]);
    assert!(listen(&harness, "camera.fov", on_fov));
    assert!(listen(&harness, "world.dimension_changing", on_dimension));
    harness
}

#[test]
fn the_listener_result_reaches_the_game() {
    let _harness = start();

    let fov = LevelRendererGame::fov(1.0, true);

    assert_eq!(fov, 15.0);
    assert_eq!(seen(), ["fov base=70 steady=60 result=70"]);
    assert_eq!(LevelRendererGame::fov_calls(), 2);
}

#[test]
fn a_call_without_variable_fov_calls_the_original_once() {
    let _harness = start();

    let fov = LevelRendererGame::fov(1.0, false);

    assert_eq!(fov, 15.0);
    assert_eq!(seen(), ["fov base=60 steady=60 result=60"]);
    assert_eq!(LevelRendererGame::fov_calls(), 1);
}

#[test]
fn without_a_listener_the_original_runs_once() {
    let (_harness, _) = install(&[]);

    assert_eq!(LevelRendererGame::fov(1.0, true), 70.0);
    assert_eq!(LevelRendererGame::fov_calls(), 1);
}

#[test]
fn a_bad_result_gives_the_game_its_own_fov() {
    let _harness = start();

    for bad in [f32::NAN, 0.0, -5.0, 180.0] {
        WRITE.store(bad.to_bits(), Ordering::SeqCst);
        assert_eq!(LevelRendererGame::fov(1.0, true), 70.0, "{bad}");
    }
    assert_eq!(seen().len(), 4);
}

#[test]
fn a_failed_listener_gives_the_game_its_own_fov() {
    let _harness = start();
    WRITE.store(10.0f32.to_bits(), Ordering::SeqCst);
    FAIL.store(1, Ordering::SeqCst);

    assert_eq!(LevelRendererGame::fov(1.0, true), 70.0);
}

#[test]
fn a_missing_entry_leaves_the_row_not_available() {
    let (harness, rows) = install(&[LevelRendererGame::GET_FOV]);

    let row = rows.iter().find(|row| row.id == "camera.fov").unwrap();
    assert_eq!((row.shape, row.missing.clone()), (None, vec![LevelRendererGame::GET_FOV]));
    assert!(!listen(&harness, "camera.fov", on_fov));
    let dimension = rows.iter().find(|row| row.id == "world.dimension_changing").unwrap();
    assert_eq!(dimension.shape, Some("default"));
    assert!(harness.state.events().is_available("world.dimension_changing", 1));
}

// Records the `_reserved` value it sees, then writes its own into it.
unsafe extern "C" fn on_fov_reserved(
    event: *mut AmmlEventHeader,
    _: *const AmmlEventContext,
    _: *mut c_void,
) -> AmmlStatus {
    // SAFETY: the loader raises `camera.fov` with a `CameraFov`.
    let fov = unsafe { &mut *event.cast::<CameraFov>() };
    SEEN.lock().unwrap().push(format!("reserved={:#x}", fov._reserved));
    fov._reserved = 0xDEAD;
    AmmlStatus::OK
}

#[test]
fn a_listener_write_to_the_reserved_word_does_not_reach_the_next_listener() {
    let (harness, _) = install(&[]);
    assert!(listen(&harness, "camera.fov", on_fov_reserved));
    assert!(listen(&harness, "camera.fov", on_fov_reserved));

    LevelRendererGame::fov(1.0, true);

    assert_eq!(seen(), ["reserved=0x0", "reserved=0x0"]);
}

#[test]
fn a_dimension_change_held_for_the_next_frame_is_dropped_when_the_world_is_left() {
    let harness = start();
    ClientGame::frame();
    ClientGame::set_player(0x5000);
    ClientGame::update(&GAME_CLIENT_A);
    assert_eq!(harness.state.events().gate().epoch().0, 1);

    std::thread::spawn(LevelRendererGame::will_change_dimension).join().unwrap();
    SessionGame::exit();
    ClientGame::frame();

    assert_eq!(dimension_events(), 0);
}

#[test]
fn the_dimension_event_comes_after_the_game() {
    let _harness = start();
    ClientGame::frame();

    LevelRendererGame::will_change_dimension();

    assert_eq!(LevelRendererGame::calls(), ["game", "dimension"]);
}

#[test]
fn a_dimension_change_off_the_game_thread_comes_at_the_next_frame() {
    let _harness = start();
    ClientGame::frame();

    std::thread::spawn(LevelRendererGame::will_change_dimension).join().unwrap();
    assert_eq!(dimension_events(), 0);
    ClientGame::frame();
    assert_eq!(dimension_events(), 1);
    ClientGame::frame();

    assert_eq!(dimension_events(), 1);
    assert_eq!(LevelRendererGame::calls(), ["game", "dimension"]);
}
