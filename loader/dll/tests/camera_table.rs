use std::ffi::c_void;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU8, AtomicUsize, Ordering};

use amml::{CameraAdapter, ClientAdapter, FreeCameraAdapter, ModListener, ModLoader, SessionAdapter};
use amml_abi::{
    AmmlApi, AmmlBand, AmmlCameraTable, AmmlEvent, AmmlEventContext, AmmlEventHeader, AmmlPhase, AmmlPose,
    AmmlProjection, AmmlRotation, AmmlServices, AmmlStatus, AmmlStr, AmmlVec3, CameraInstructionReceived, CameraSetup,
};
use amml_runtime::ModRegistry;

use crate::support::{CLIENT_A, CameraGame, EventsMod, FreeCameraGame, Harness, LogFile, SessionGame};

// The module of the mod that `start` loads.
const MODULE: &str = "@events/events";
const POSE: AmmlPose = AmmlPose {
    position: AmmlVec3 { x: 12.5, y: 70.0, z: -3.25 },
    rotation: AmmlRotation { yaw: 90.0, pitch: 0.0 },
    roll: 0.0,
};

// What the listener does in each call.
const SET_POSE: u8 = 1;
const CHECK_STATUSES: u8 = 2;
const READ_ONLY: u8 = 3;
const GAME_CALL: u8 = 4;
const NOTHING: u8 = 5;

// What the `camera.setup` listener saw, one line per call.
static SEEN: Mutex<Vec<String>> = Mutex::new(Vec::new());
// The frame id of the last `CameraSetup` call.
static LAST_FRAME: AtomicUsize = AtomicUsize::new(0);
static MODE: AtomicU8 = AtomicU8::new(0);
// The address of the loader's camera table, for the listener.
static TABLE: AtomicUsize = AtomicUsize::new(0);

fn table() -> &'static AmmlCameraTable {
    // SAFETY: `TABLE` holds the address of the loader's camera table, which lives for the process.
    unsafe { &*std::ptr::with_exposed_provenance::<AmmlCameraTable>(TABLE.load(Ordering::SeqCst)) }
}

fn set_pose(frame: u64, pose: &AmmlPose) -> AmmlStatus {
    // SAFETY: the table is live; the module and the pose live for the call.
    unsafe { (table().set_pose)(AmmlStr::from_str(MODULE), frame, std::ptr::from_ref(pose)) }
}

fn rendered(frame: u64) -> (AmmlStatus, AmmlVec3) {
    let mut out = AmmlVec3::default();
    // SAFETY: the table is live; `out` is writable for the call.
    let status = unsafe { (table().rendered_position)(frame, &raw mut out) };
    (status, out)
}

fn projection(frame: u64) -> (AmmlStatus, AmmlProjection) {
    let mut out = AmmlProjection::default();
    // SAFETY: the table is live; `out` is writable for the call.
    let status = unsafe { (table().projection)(frame, &raw mut out) };
    (status, out)
}

unsafe extern "C" fn on_setup(
    event: *mut AmmlEventHeader,
    context: *const AmmlEventContext,
    _: *mut c_void,
) -> AmmlStatus {
    // SAFETY: the loader raises `camera.setup` with a `CameraSetup` and a live context.
    let (setup, context) = unsafe { (&*event.cast::<CameraSetup>(), &*context) };
    let frame = setup.frame_id;
    LAST_FRAME.store(frame as usize, Ordering::SeqCst);
    let phase = if setup.header.phase == AmmlPhase::BEFORE { "before" } else { "after" };
    let mut line = format!("{phase} frame={frame} first={} thread={}", setup.first_in_frame, context.thread.0);
    match MODE.load(Ordering::SeqCst) {
        SET_POSE if setup.header.phase == AmmlPhase::BEFORE => {
            line.push_str(&format!(" set={}", set_pose(frame, &POSE).0));
        }
        CHECK_STATUSES if setup.header.phase == AmmlPhase::BEFORE => {
            let nan = AmmlPose { roll: f32::NAN, ..POSE };
            let infinite = AmmlPose { position: AmmlVec3 { x: f32::INFINITY, ..POSE.position }, ..POSE };
            // SAFETY: the table is live; a null pose is refused before it is read.
            let null = unsafe { (table().set_pose)(AmmlStr::from_str(MODULE), frame, std::ptr::null()) };
            let other_thread = std::thread::spawn(move || set_pose(frame, &POSE)).join().unwrap();
            line.push_str(&format!(
                " nan={} infinite={} null={} next={} zero={} other_thread={}",
                set_pose(frame, &nan).0,
                set_pose(frame, &infinite).0,
                null.0,
                set_pose(frame + 1, &POSE).0,
                set_pose(0, &POSE).0,
                other_thread.0
            ));
        }
        READ_ONLY => {
            let (status, position) = rendered(frame);
            let (projection_status, values) = projection(frame);
            line.push_str(&format!(
                " rendered={} {},{},{} projection={} {:.1},{:.3}",
                status.0, position.x, position.y, position.z, projection_status.0, values.fov_y, values.aspect
            ));
            if setup.header.phase == AmmlPhase::BEFORE {
                line.push_str(&format!(" set={}", set_pose(frame, &POSE).0));
            }
        }
        GAME_CALL => {
            let allowed = match CameraAdapter::check_game_call() {
                Ok(()) => 0,
                Err(status) => status.0,
            };
            line.push_str(&format!(" game_call={allowed}"));
        }
        _ => {}
    }
    SEEN.lock().unwrap().push(line);
    AmmlStatus::OK
}

fn seen() -> Vec<String> {
    SEEN.lock().unwrap().clone()
}

// A perspective projection with fov y 70 degrees and aspect 16 / 9.
fn perspective() -> [[f32; 4]; 4] {
    let focal_y = 1.0 / (35.0f32).to_radians().tan();
    let mut matrix = [[0.0f32; 4]; 4];
    matrix[0][0] = focal_y / (16.0 / 9.0);
    matrix[1][1] = focal_y;
    matrix
}

// The session and camera adapters over the stand-ins with `entries`, one loaded mod, its table found, and a
// `camera.setup` listener of that mod in `mode` (none for 0).
fn start_with(entries: &[(&'static str, amml_data::Resolved)], mode: u8) -> Harness {
    let harness = Harness::start_with_entries(entries);
    SessionAdapter::install(harness.state);
    CameraAdapter::install(harness.state);
    let plain = EventsMod::build("plain", &[]);
    harness.install_mod("events", "events", &plain, "events.dll", 3);
    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    assert!(ModLoader::new(harness.state).load_all(&scan.mods).iter().all(|report| report.is_ok()));
    // SAFETY: the loader's tables live for the process.
    let services = unsafe { AmmlApi::services(harness.state.api()) }.unwrap();
    // SAFETY: as above.
    let camera = unsafe { AmmlServices::camera(services) }.unwrap();
    TABLE.store(camera.expose_provenance(), Ordering::SeqCst);
    MODE.store(mode, Ordering::SeqCst);
    if mode != 0 {
        let listener = ModListener { owner: 0, callback: on_setup, user: 0 };
        assert!(harness.state.events().listen("camera.setup", 1, AmmlBand::NORMAL, listener).is_ok());
    }
    CameraGame::reset(perspective());
    harness
}

fn start(mode: u8) -> Harness {
    start_with(&CameraGame::entries(&[], &[]), mode)
}

fn join() {
    SessionGame::frame();
    SessionGame::set_player(0x5000);
    SessionGame::start_join(&CLIENT_A, [0; 7]);
    SessionGame::update(&CLIENT_A);
}

// One game frame: `FrameUpdate`, then `setupCamera`, `createViewRenderObject` and `preRenderUpdate`.
fn frame() {
    SessionGame::frame();
    CameraGame::setup(0.5);
    assert_eq!(CameraGame::create(), CameraGame::view_object());
    CameraGame::pre_render();
}

#[test]
fn set_pose_outside_the_dispatch_is_refused() {
    let _harness = start(CHECK_STATUSES);
    join();

    frame();

    let line = &seen()[0];
    let refused = AmmlStatus::REFUSED.0;
    assert!(line.contains(&format!("next={refused} zero={refused} other_thread={refused}")), "{line}");
    // After the dispatch its own frame is refused.
    let frame = LAST_FRAME.load(Ordering::SeqCst) as u64;
    assert_eq!(set_pose(frame, &POSE), AmmlStatus::REFUSED);
    assert_eq!(rendered(frame).0, AmmlStatus::REFUSED);
    assert_eq!(projection(frame).0, AmmlStatus::REFUSED);
}

#[test]
fn non_finite_pose_is_invalid_argument() {
    let _harness = start(CHECK_STATUSES);
    join();

    frame();

    let line = &seen()[0];
    let invalid = AmmlStatus::INVALID_ARGUMENT.0;
    assert!(line.contains(&format!("nan={invalid} infinite={invalid} null={invalid}")), "{line}");
    // Nothing was set, so the game camera shows.
    assert_eq!(CameraGame::view(), [CameraGame::GAME_POSITION; 2]);
    assert!(!CameraGame::log().contains(&"update".to_string()));
}

#[test]
fn a_pose_is_written_after_the_setup_the_view_object_and_the_pre_render_update() {
    let _harness = start(SET_POSE);
    join();

    frame();

    let position = [POSE.position.x, POSE.position.y, POSE.position.z];
    assert_eq!(CameraGame::view(), [position; 2]);
    assert_eq!(CameraGame::level(), [position; 2]);
    // Yaw 90 looks along -X; the view matrix rows are right, up and minus forward.
    let forward = CameraGame::forward();
    assert!((forward[0] + 1.0).abs() < 1e-6 && forward[1].abs() < 1e-6 && forward[2].abs() < 1e-6, "{forward:?}");
    let view = CameraGame::view_top();
    assert!((view[0][2] - 1.0).abs() < 1e-6, "{view:?}");
    assert!(CameraGame::view_dirty());
    // The originals ran once each with their own arguments, then the loader called the view update.
    assert_eq!(CameraGame::log(), ["setup alpha=0.5", "update", "create sub=1", "pre_render"]);
    let lines = seen();
    assert_eq!(lines.len(), 2, "{lines:?}");
    assert!(lines[0].starts_with("before") && lines[0].contains("first=1 thread=0 set=0"), "{lines:?}");
    assert!(lines[1].starts_with("after") && lines[1].contains("first=1"), "{lines:?}");
}

#[test]
fn a_frame_without_set_pose_writes_nothing() {
    let _harness = start(SET_POSE);
    join();
    frame();

    MODE.store(NOTHING, Ordering::SeqCst);
    CameraGame::reset(perspective());
    frame();

    assert_eq!(CameraGame::view(), [CameraGame::GAME_POSITION; 2]);
    assert_eq!(CameraGame::level(), [CameraGame::GAME_POSITION; 2]);
    assert!(!CameraGame::view_dirty());
    assert_eq!(CameraGame::log(), ["setup alpha=0.5", "create sub=1", "pre_render"]);
}

#[test]
fn rendered_position_and_projection_answer_in_the_dispatch() {
    let _harness = start(READ_ONLY);
    join();

    frame();
    frame();

    let lines = seen();
    let failed = AmmlStatus::FAILED.0;
    let refused = AmmlStatus::REFUSED.0;
    // Frame 1: nothing written before it; the projection is read in After only.
    assert!(lines[0].contains(&format!("rendered={failed} 0,0,0 projection={refused}")), "{lines:?}");
    assert!(lines[1].contains("projection=0 70.0,1.778"), "{lines:?}");
    // Frame 2: the position the loader wrote in frame 1.
    assert!(lines[2].contains("rendered=0 12.5,70,-3.25"), "{lines:?}");
}

#[test]
fn a_second_setup_in_one_frame_is_not_first() {
    let _harness = start(SET_POSE);
    join();

    SessionGame::frame();
    CameraGame::setup(0.5);
    CameraGame::setup(0.25);

    let lines = seen();
    assert!(lines[0].contains("first=1") && lines[2].contains("first=0"), "{lines:?}");
    assert_eq!(lines[0].split(' ').nth(1), lines[2].split(' ').nth(1));
}

#[test]
fn without_a_listener_or_a_pose_the_originals_run_and_nothing_is_written() {
    let _harness = start(0);
    join();

    frame();

    assert!(seen().is_empty());
    assert_eq!(CameraGame::log(), ["setup alpha=0.5", "create sub=1", "pre_render"]);
    assert_eq!(CameraGame::view(), [CameraGame::GAME_POSITION; 2]);
}

#[test]
fn a_setup_off_the_game_thread_refuses_game_calls_and_logs_once() {
    let harness = start(GAME_CALL);
    join();
    SessionGame::frame();

    std::thread::spawn(|| {
        CameraGame::setup(0.5);
        CameraGame::setup(0.5);
    })
    .join()
    .unwrap();
    CameraGame::setup(0.5);

    let lines = seen();
    let refused = AmmlStatus::REFUSED.0;
    assert!(lines[0].contains(&format!("thread=1 game_call={refused}")), "{lines:?}");
    assert!(lines[4].contains("thread=0 game_call=0"), "{lines:?}");
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "camera setup thread {\"game\":true}");
    assert_eq!(log.matches("camera setup off the game thread").count(), 1, "{log}");
    assert!(log.contains("camera setup thread {\"game\":false}"), "{log}");
    // A game call outside any camera dispatch is not judged here.
    assert_eq!(CameraAdapter::check_game_call(), Ok(()));
}

#[test]
fn a_camera_layout_mismatch_offers_setup_but_not_pose() {
    let entries = CameraGame::entries(&[], &[("Camera.mForward", 0x120)]);
    let harness = start_with(&entries, SET_POSE);
    join();

    frame();

    // The pose area is not offered: `set_pose` gives NOT_AVAILABLE and nothing is written.
    let lines = seen();
    assert!(lines[0].contains(&format!("set={}", AmmlStatus::NOT_AVAILABLE.0)), "{lines:?}");
    assert_eq!(CameraGame::view(), [CameraGame::GAME_POSITION; 2]);
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "camera layout mismatch");
    assert!(log.contains("Camera.mForward"), "{log}");
    assert!(harness.state.events().is_available("camera.setup", 1), "{log}");
    assert!(!harness.state.areas().is_available("camera.pose", 1));
    assert!(!harness.state.areas().is_available("camera.projection", 1));
}

#[test]
fn disable_drops_the_pose_of_the_mod() {
    let _harness = start(SET_POSE);
    join();
    SessionGame::frame();
    CameraGame::setup(0.5);

    CameraAdapter::get().unwrap().disable_owner(0);
    assert_eq!(CameraGame::create(), CameraGame::view_object());

    assert_eq!(CameraGame::view(), [CameraGame::GAME_POSITION; 2]);
}

// What the `camera.instruction_received` listener saw, one line per call.
static INSTRUCTIONS: Mutex<Vec<String>> = Mutex::new(Vec::new());

unsafe extern "C" fn on_instruction(
    event: *mut AmmlEventHeader,
    context: *const AmmlEventContext,
    _: *mut c_void,
) -> AmmlStatus {
    // SAFETY: the loader raises `camera.instruction_received` with a `CameraInstructionReceived` and a live context.
    let (received, context) = unsafe { (&*event.cast::<CameraInstructionReceived>(), &*context) };
    let header = received.header;
    INSTRUCTIONS.lock().unwrap().push(format!(
        "kind={} origin={} injected={} phase={} thread={}",
        received.kind.0, header.origin.0, header.injected, header.phase.0, context.thread.0
    ));
    AmmlStatus::OK
}

fn instructions() -> Vec<String> {
    INSTRUCTIONS.lock().unwrap().clone()
}

fn set_free(pose: &AmmlPose) -> AmmlStatus {
    // SAFETY: the table is live; the module and the pose live for the call.
    unsafe { (table().set_free.unwrap())(AmmlStr::from_str(MODULE), std::ptr::from_ref(pose)) }
}

fn release_free() -> AmmlStatus {
    // SAFETY: the table is live; the module lives for the call.
    unsafe { (table().release_free.unwrap())(AmmlStr::from_str(MODULE)) }
}

fn free_available() -> (AmmlStatus, u8) {
    let mut out = 9u8;
    // SAFETY: the table is live; `out` is writable for the call.
    let status = unsafe { (table().free_available.unwrap())(&raw mut out) };
    (status, out)
}

// The session, client, camera and free camera adapters over the free camera stand-ins, the level's presets `names`,
// one mod enabled (declaring `camera.free` when `declares`) with its table found and a `camera.instruction_received`
// listener, and a joined world.
fn start_free(names: &[&str], declares: bool) -> Harness {
    FreeCameraGame::reset(names);
    let harness = Harness::start_with_entries(&FreeCameraGame::entries());
    SessionAdapter::install(harness.state);
    ClientAdapter::install(harness.state);
    CameraAdapter::install(harness.state);
    FreeCameraAdapter::install(harness.state);
    let dll = if declares { EventsMod::build("free-camera", &["free-camera"]) } else { EventsMod::build("plain", &[]) };
    harness.install_mod("events", "events", &dll, "events.dll", 3);
    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    let loader = ModLoader::new(harness.state);
    assert!(loader.load_all(&scan.mods).iter().all(|report| report.is_ok()));
    assert!(loader.enable_all().iter().all(|report| report.is_ok()));
    // SAFETY: the loader's tables live for the process.
    let services = unsafe { AmmlApi::services(harness.state.api()) }.unwrap();
    // SAFETY: as above.
    let camera = unsafe { AmmlServices::camera(services) }.unwrap();
    TABLE.store(camera.expose_provenance(), Ordering::SeqCst);
    let listener = ModListener { owner: 0, callback: on_instruction, user: 0 };
    assert!(harness.state.events().listen(CameraInstructionReceived::ID, 1, AmmlBand::NORMAL, listener).is_ok());
    SessionGame::frame();
    SessionGame::set_player(FreeCameraGame::player());
    SessionGame::start_join(&CLIENT_A, [0; 7]);
    SessionGame::update(&CLIENT_A);
    SessionGame::frame();
    harness
}

#[test]
fn set_free_before_capture_is_not_ready() {
    let harness = start_free(&["minecraft:free"], true);
    assert!(harness.state.areas().is_available("camera.free", 1));
    assert!(harness.state.events().is_available(CameraInstructionReceived::ID, 1));

    assert_eq!(set_free(&POSE), AmmlStatus::NOT_READY);
    assert_eq!(release_free(), AmmlStatus::NOT_READY);

    // Nothing was made or handed to the game.
    assert!(FreeCameraGame::created().is_empty());
    assert!(FreeCameraGame::log().is_empty());
    // The preset list is read without a captured handler.
    assert_eq!(free_available(), (AmmlStatus::OK, 1));
}

#[test]
fn set_free_off_the_game_thread_is_refused() {
    let _harness = start_free(&["minecraft:free"], true);
    FreeCameraGame::chunk_publisher();

    let statuses = std::thread::spawn(|| (set_free(&POSE), release_free(), free_available().0)).join().unwrap();

    let refused = AmmlStatus::REFUSED;
    assert_eq!(statuses, (refused, refused, refused));
    assert!(FreeCameraGame::created().is_empty());
}

#[test]
fn set_free_after_capture_hands_the_set_instruction_to_the_captured_handler() {
    let _harness = start_free(&["a:b", "minecraft:free"], true);
    FreeCameraGame::chunk_publisher();

    assert_eq!(set_free(&POSE), AmmlStatus::OK);
    assert_eq!(release_free(), AmmlStatus::OK);

    // One CameraInstructionPacket (id 300) serves both deliveries.
    assert_eq!(FreeCameraGame::created(), [300]);
    assert_eq!(FreeCameraGame::uses(), FreeCameraGame::USES + 1);
    let handler = FreeCameraGame::handler();
    let id = FreeCameraGame::ID;
    let log = FreeCameraGame::log();
    assert_eq!(log.len(), 3, "{log:?}");
    assert_eq!(log[0], format!("chunk handler={handler:#x} id={id:#x}"));
    // The free preset is the second of the list; the rotation is pitch, then yaw.
    let set = "set=1 index=1 pos=12.5,70,-3.25/1 rot=0,90/1 clear=0/0";
    // The identifier is the loader's own zeroed buffer, not the address of an earlier call.
    assert_eq!(log[1], format!("camera handler={handler:#x} id=invalid {set}"));
    let clear = "set=0 index=0 pos=0,0,0/0 rot=0,0/0 clear=1/1";
    assert_eq!(log[2], format!("camera handler={handler:#x} id=invalid {clear}"));
    // The mod's own instructions reach its listener as THIS_MOD and injected, Before the game's handler.
    let seen = instructions();
    assert_eq!(seen, ["kind=1 origin=1 injected=1 phase=0 thread=0", "kind=2 origin=1 injected=1 phase=0 thread=0"]);
}

#[test]
fn a_server_instruction_is_raised_with_origin_game_and_captures_the_handler() {
    let _harness = start_free(&["minecraft:free"], true);

    FreeCameraGame::server_instruction(true, false);
    FreeCameraGame::server_instruction(false, true);
    FreeCameraGame::server_instruction(true, true);
    FreeCameraGame::server_instruction(false, false);

    assert_eq!(
        instructions(),
        [
            "kind=1 origin=0 injected=0 phase=0 thread=0",
            "kind=2 origin=0 injected=0 phase=0 thread=0",
            "kind=3 origin=0 injected=0 phase=0 thread=0",
            "kind=3 origin=0 injected=0 phase=0 thread=0",
        ]
    );
    // The camera handler call captured the handler.
    assert_eq!(set_free(&POSE), AmmlStatus::OK);
}

#[test]
fn without_a_mod_that_declares_the_area_the_handler_is_not_captured() {
    let _harness = start_free(&["minecraft:free"], false);

    FreeCameraGame::chunk_publisher();

    assert_eq!(set_free(&POSE), AmmlStatus::NOT_READY);
}

#[test]
fn a_level_without_the_free_preset_is_not_available() {
    let _harness = start_free(&["a:b"], true);
    FreeCameraGame::chunk_publisher();

    assert_eq!(set_free(&POSE), AmmlStatus::NOT_AVAILABLE);
    assert_eq!(free_available(), (AmmlStatus::OK, 0));
    assert!(FreeCameraGame::created().is_empty());
}

#[test]
fn a_non_finite_pose_is_invalid_argument() {
    let _harness = start_free(&["minecraft:free"], true);
    FreeCameraGame::chunk_publisher();

    let nan = AmmlPose { roll: f32::NAN, ..POSE };
    assert_eq!(set_free(&nan), AmmlStatus::INVALID_ARGUMENT);
    // SAFETY: the table is live; a null pose is refused before it is read.
    let null = unsafe { (table().set_free.unwrap())(AmmlStr::from_str(MODULE), std::ptr::null()) };
    assert_eq!(null, AmmlStatus::INVALID_ARGUMENT);
}

#[test]
fn disable_sends_the_clear_and_gives_the_packet_back() {
    let _harness = start_free(&["minecraft:free"], true);
    FreeCameraGame::chunk_publisher();
    assert_eq!(set_free(&POSE), AmmlStatus::OK);

    FreeCameraAdapter::get().unwrap().disable_owner(0);

    let log = FreeCameraGame::log();
    assert!(log.last().unwrap().ends_with("set=0 index=0 pos=0,0,0/0 rot=0,0/0 clear=1/1"), "{log:?}");
    assert_eq!(FreeCameraGame::uses(), FreeCameraGame::USES);
}

#[test]
fn world_leave_gives_the_packet_back_and_forgets_the_handler() {
    let _harness = start_free(&["minecraft:free"], true);
    FreeCameraGame::chunk_publisher();
    assert_eq!(set_free(&POSE), AmmlStatus::OK);
    let delivered = FreeCameraGame::log().len();

    SessionGame::exit();

    // No clear goes to the handler of a left world.
    assert_eq!(FreeCameraGame::log().len(), delivered);
    assert_eq!(FreeCameraGame::uses(), FreeCameraGame::USES);
    assert_eq!(set_free(&POSE), AmmlStatus::NOT_READY);
}

#[test]
fn a_shutdown_disable_sends_nothing_and_runs_no_game_code_for_the_packet() {
    let harness = start_free(&["minecraft:free"], true);
    FreeCameraGame::chunk_publisher();
    assert_eq!(set_free(&POSE), AmmlStatus::OK);
    let delivered = FreeCameraGame::log().len();

    ModLoader::new(harness.state).shutdown();

    assert_eq!(FreeCameraGame::log().len(), delivered);
    assert_eq!(FreeCameraGame::uses(), FreeCameraGame::USES + 1);
}

// The mod holds the free camera, then the server sends an instruction with its set part when `set` and its clear part
// when `clear`: the old owner's disable sends no clear over the server's camera, and the mod may take it again.
fn server_takes_the_camera(set: bool, clear: bool) {
    let _harness = start_free(&["minecraft:free"], true);
    FreeCameraGame::chunk_publisher();
    assert_eq!(set_free(&POSE), AmmlStatus::OK);

    FreeCameraGame::server_instruction(set, clear);
    let delivered = FreeCameraGame::log().len();
    FreeCameraAdapter::get().unwrap().disable_owner(0);

    let log = FreeCameraGame::log();
    assert_eq!(log.len(), delivered, "{log:?}");
    assert_eq!(set_free(&POSE), AmmlStatus::OK);
}

#[test]
fn a_server_set_forgets_the_free_camera_owner() {
    server_takes_the_camera(true, false);
}

#[test]
fn a_server_clear_forgets_the_free_camera_owner() {
    server_takes_the_camera(false, true);
}

#[test]
fn a_server_instruction_without_a_set_or_clear_keeps_the_owner() {
    let _harness = start_free(&["minecraft:free"], true);
    FreeCameraGame::chunk_publisher();
    assert_eq!(set_free(&POSE), AmmlStatus::OK);

    FreeCameraGame::server_instruction(false, false);
    FreeCameraAdapter::get().unwrap().disable_owner(0);

    let log = FreeCameraGame::log();
    assert!(log.last().unwrap().ends_with("clear=1/1"), "{log:?}");
}

#[test]
fn a_disable_whose_clear_cannot_reach_the_handler_logs_a_warning() {
    let harness = start_free(&["minecraft:free"], true);
    FreeCameraGame::chunk_publisher();
    assert_eq!(set_free(&POSE), AmmlStatus::OK);
    let delivered = FreeCameraGame::log().len();

    // A join with no world leave before it forgets the handler; the holder's disable then has no handler to clear.
    SessionGame::start_join(&CLIENT_A, [0; 7]);
    FreeCameraAdapter::get().unwrap().disable_owner(0);

    assert_eq!(FreeCameraGame::log().len(), delivered);
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "the clear of a disabled mod's free camera");
    assert!(log.contains("could not reach the client packet handler"), "{log}");
}
