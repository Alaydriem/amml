use std::ffi::c_void;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use amml::{ClientAdapter, InputAdapter, ModListener, MoveLayout, SessionAdapter};
use amml_abi::{
    AmmlBand, AmmlEventContext, AmmlEventHeader, AmmlInputBlocks, AmmlLeaseMode, AmmlMoveIntent, AmmlStatus,
    GamepadPoll, LookDelta, PlayerMoveInput,
};
use amml_data::Availability;
use amml_runtime::{LeaseTerms, ResourceId};

use crate::support::{ClientGame, GAME_CLIENT_A, Harness, LookMoveGame};

const PLAYER: usize = 0x5000;
const UP_BIT: u32 = 1 << 13;

// What the listeners saw, one line per event.
static SEEN: Mutex<Vec<String>> = Mutex::new(Vec::new());
// When set, the `input.look_delta` listener halves the yaw.
static HALVE_YAW: AtomicBool = AtomicBool::new(false);
// When set, the `input.player_move` listener sets the left axis to 1.
static MOVE_LEFT: AtomicBool = AtomicBool::new(false);
// When set, the `input.gamepad_poll` listener cancels the poll.
static CANCEL_POLL: AtomicBool = AtomicBool::new(false);

unsafe extern "C" fn on_look(event: *mut AmmlEventHeader, _: *const AmmlEventContext, _: *mut c_void) -> AmmlStatus {
    // SAFETY: the loader raises `input.look_delta` with a `LookDelta`.
    let look = unsafe { &mut *event.cast::<LookDelta>() };
    SEEN.lock().unwrap().push(format!(
        "look yaw={} pitch={} cancelled={} injected={} origin={}",
        look.yaw, look.pitch, look.header.cancelled, look.header.injected, look.header.origin.0
    ));
    if HALVE_YAW.load(Ordering::SeqCst) {
        look.yaw /= 2.0;
    }
    AmmlStatus::OK
}

unsafe extern "C" fn on_move(event: *mut AmmlEventHeader, _: *const AmmlEventContext, _: *mut c_void) -> AmmlStatus {
    // SAFETY: the loader raises `input.player_move` with a `PlayerMoveInput`.
    let movement = unsafe { &mut *event.cast::<PlayerMoveInput>() };
    let intent = movement.intent;
    SEEN.lock().unwrap().push(format!(
        "move forward={} left={} jump={} sneak={} sprint={} injected={}",
        intent.forward, intent.left, intent.jump, intent.sneak, intent.sprint, movement.header.injected
    ));
    if MOVE_LEFT.load(Ordering::SeqCst) {
        movement.intent.left = 1.0;
    }
    AmmlStatus::OK
}

unsafe extern "C" fn on_poll(event: *mut AmmlEventHeader, _: *const AmmlEventContext, _: *mut c_void) -> AmmlStatus {
    // SAFETY: the loader raises `input.gamepad_poll` with a `GamepadPoll`.
    let poll = unsafe { &mut *event.cast::<GamepadPoll>() };
    SEEN.lock().unwrap().push(format!("poll cancelled={}", poll.header.cancelled));
    if CANCEL_POLL.load(Ordering::SeqCst) {
        poll.header.cancelled = 1;
    }
    AmmlStatus::OK
}

fn seen() -> Vec<String> {
    SEEN.lock().unwrap().clone()
}

// Installs the session, client and input adapters over the stand-ins with `layout` changed, and listens to
// the three events as mod 0; gives the harness and the input rows.
fn install(layout: &[(&'static str, u64)]) -> (Harness, Vec<Availability>) {
    let harness = Harness::start_with_entries(&LookMoveGame::entries(layout));
    SessionAdapter::install(harness.state);
    ClientAdapter::install(harness.state);
    let rows = InputAdapter::install(harness.state);
    let events = harness.state.events();
    let listen = |id: &str, callback| {
        events.listen(id, 1, AmmlBand::NORMAL, ModListener { owner: 0, callback, user: 0 }).ok();
    };
    listen("input.look_delta", on_look);
    listen("input.player_move", on_move);
    listen("input.gamepad_poll", on_poll);
    (harness, rows)
}

// `install` with the 26.63 layout, then one frame, an update of client A (the primary client) and a joined
// world whose local player is `PLAYER`.
fn start() -> Harness {
    let (harness, _) = install(&[]);
    ClientGame::frame();
    ClientGame::update(&GAME_CLIENT_A);
    ClientGame::set_player(PLAYER);
    ClientGame::update(&GAME_CLIENT_A);
    harness
}

fn lease(harness: &Harness, blocks: AmmlInputBlocks) -> ResourceId {
    let terms = LeaseTerms { mode: AmmlLeaseMode::SHARED, blocks, passthrough: Vec::new(), preempt: false };
    harness.state.input().acquire(0, "@test/mod0", terms).unwrap()
}

fn forward() -> AmmlMoveIntent {
    AmmlMoveIntent { forward: 1.0, left: 0.0, jump: 0, sneak: 0, sprint: 0, _reserved: 0 }
}

fn float_at(raw: &[u8; 0x18], at: usize) -> f32 {
    f32::from_le_bytes(raw[at..at + 4].try_into().unwrap())
}

fn flags(raw: &[u8; 0x18]) -> u32 {
    u32::from_le_bytes(raw[0..4].try_into().unwrap())
}

// The analog vector (MoveInputState+4) and the raw move (+10h) of the 26.63 layout, as [left, forward].
fn vectors(raw: &[u8; 0x18]) -> ([f32; 2], [f32; 2]) {
    ([float_at(raw, 4), float_at(raw, 8)], [float_at(raw, 0x10), float_at(raw, 0x14)])
}

// A component as the game fills it while the user holds forward: the Up bit, and edge bit 1.
fn keyboard_forward() -> [u8; 0x18] {
    let mut fill = [0u8; 0x18];
    fill[0..4].copy_from_slice(&(UP_BIT | 2).to_le_bytes());
    fill
}

#[test]
fn a_listener_can_scale_the_look_delta() {
    let _harness = start();
    HALVE_YAW.store(true, Ordering::SeqCst);
    let delta = [2.0f32, 10.0];

    LookMoveGame::turn(PLAYER, &delta);

    // The game's x grows looking up; the API pitch grows looking down.
    assert_eq!(seen(), ["look yaw=10 pitch=-2 cancelled=0 injected=0 origin=0"]);
    assert_eq!(LookMoveGame::calls(), ["turn this=0x5000 x=2 y=5"]);
    assert_eq!(delta, [2.0, 10.0]);
}

#[test]
fn a_look_lease_blocks_mouse_look() {
    let harness = start();
    lease(&harness, AmmlInputBlocks::LOOK);

    LookMoveGame::turn(PLAYER, &[2.0, 10.0]);

    assert_eq!(seen(), ["look yaw=10 pitch=-2 cancelled=1 injected=0 origin=0"]);
    assert!(LookMoveGame::calls().is_empty());
}

#[test]
fn turns_of_one_frame_are_applied_once() {
    let harness = start();
    let input = harness.state.input();
    let lease = lease(&harness, AmmlInputBlocks::LOOK);

    input.turn(lease, 10.0, 0.0).unwrap();
    input.turn(lease, 10.0, 0.0).unwrap();
    assert!(LookMoveGame::calls().is_empty());
    ClientGame::frame();

    // The game's Vec2 is [pitch, yaw] in degrees: yaw 20 is y 20.
    assert_eq!(LookMoveGame::calls(), ["turn this=0x5000 x=0 y=20"]);
    // The summed turn has no single injector, so its own mod sees OTHER_MOD.
    assert_eq!(seen(), ["look yaw=20 pitch=0 cancelled=0 injected=1 origin=2"]);
}

#[test]
fn an_injected_pitch_down_reaches_the_game_as_a_negative_x() {
    let harness = start();
    let input = harness.state.input();
    let lease = lease(&harness, AmmlInputBlocks::LOOK);

    input.turn(lease, 0.0, 5.0).unwrap();
    ClientGame::frame();

    assert_eq!(LookMoveGame::calls(), ["turn this=0x5000 x=-5 y=0"]);
    assert_eq!(seen(), ["look yaw=0 pitch=5 cancelled=0 injected=1 origin=2"]);
}

#[test]
fn the_move_intent_is_written_before_listeners_see_it() {
    let harness = start();
    let input = harness.state.input();
    let lease = lease(&harness, AmmlInputBlocks(0));
    let mut raw = [0u8; 0x18];

    input.set_move(lease, Some(forward())).unwrap();
    LookMoveGame::extract(&mut raw);

    assert_eq!(LookMoveGame::calls(), ["extract 1 2 3 5 6"]);
    assert_eq!(seen(), ["move forward=1 left=0 jump=0 sneak=0 sprint=0 injected=1"]);
    assert_eq!(vectors(&raw), ([0.0, 1.0], [0.0, 1.0]));
    assert_eq!(flags(&raw), UP_BIT);
}

#[test]
fn a_move_lease_zeroes_real_movement() {
    let harness = start();
    let mut fill = keyboard_forward();
    fill[8..12].copy_from_slice(&1.0f32.to_le_bytes());
    LookMoveGame::set_fill(fill);
    let mut raw = [0u8; 0x18];
    LookMoveGame::extract(&mut raw);
    lease(&harness, AmmlInputBlocks::MOVE);

    LookMoveGame::extract(&mut raw);

    assert_eq!(
        seen(),
        [
            "move forward=1 left=0 jump=0 sneak=0 sprint=0 injected=0",
            "move forward=0 left=0 jump=0 sneak=0 sprint=0 injected=0",
        ]
    );
    assert_eq!(vectors(&raw), ([0.0, 0.0], [0.0, 0.0]));
    // The edge bit stays as the game set it.
    assert_eq!(flags(&raw), 2);
}

#[test]
fn a_listener_can_change_the_move_intent() {
    let _harness = start();
    LookMoveGame::set_fill(keyboard_forward());
    MOVE_LEFT.store(true, Ordering::SeqCst);
    let mut raw = [0u8; 0x18];

    LookMoveGame::extract(&mut raw);

    assert_eq!(seen(), ["move forward=1 left=0 jump=0 sneak=0 sprint=0 injected=0"]);
    // The changed intent is written: left 1, forward 1, held to length 1, with both direction bits.
    let half = std::f32::consts::FRAC_1_SQRT_2;
    assert_eq!(vectors(&raw), ([half, half], [half, half]));
    assert_eq!(flags(&raw), UP_BIT | (1 << 15) | 2);
}

#[test]
fn a_bad_move_layout_makes_move_not_available() {
    let (harness, rows) = install(&[("RawMoveInputComponent.mRawMove", 0x14)]);
    ClientGame::frame();
    let input = harness.state.input();
    let lease = lease(&harness, AmmlInputBlocks(0));
    let mut raw = [0u8; 0x18];

    assert_eq!(input.set_move(lease, Some(forward())), Err(AmmlStatus::NOT_AVAILABLE));
    LookMoveGame::extract(&mut raw);

    let row = rows.iter().find(|row| row.id == "input.move").unwrap();
    assert_eq!((row.shape, row.missing.clone()), (None, MoveLayout::ENTRIES.to_vec()));
    assert!(!harness.state.areas().is_available("input.move", 1));
    assert!(!harness.state.events().is_available("input.player_move", 1));
    assert!(seen().is_empty());
    assert_eq!(raw, [0u8; 0x18]);
}

#[test]
fn a_cancelled_gamepad_poll_skips_the_refresh() {
    let _harness = start();

    LookMoveGame::refresh_poll();
    CANCEL_POLL.store(true, Ordering::SeqCst);
    LookMoveGame::refresh_poll();

    assert_eq!(seen(), ["poll cancelled=0", "poll cancelled=0"]);
    assert_eq!(LookMoveGame::calls(), ["refresh this=0x9a0000"]);
}

#[test]
fn a_gamepad_lease_skips_the_refresh() {
    let harness = start();
    lease(&harness, AmmlInputBlocks::GAMEPAD);

    LookMoveGame::refresh_poll();

    assert_eq!(seen(), ["poll cancelled=1"]);
    assert!(LookMoveGame::calls().is_empty());
}

#[test]
fn the_look_move_and_gamepad_rows_are_offered() {
    let (harness, rows) = install(&[]);

    for id in ["input.look_delta", "input.turn", "input.player_move", "input.move", "input.gamepad_poll"] {
        let row = rows.iter().find(|row| row.id == id).unwrap();
        assert_eq!(row.shape, Some("default"), "{id}");
    }
    assert!(harness.state.areas().is_available("input.turn", 1));
    assert!(harness.state.areas().is_available("input.move", 1));
    assert!(harness.state.events().is_available("input.gamepad_poll", 1));
}

#[test]
fn a_turn_needs_no_client_read_area() {
    let entries: Vec<_> =
        LookMoveGame::entries(&[]).into_iter().filter(|(id, _)| *id != "ClientInstance.isWorldActive").collect();
    let harness = Harness::start_with_entries(&entries);
    SessionAdapter::install(harness.state);
    ClientAdapter::install(harness.state);
    InputAdapter::install(harness.state);
    ClientGame::frame();
    ClientGame::update(&GAME_CLIENT_A);
    ClientGame::set_player(PLAYER);
    ClientGame::update(&GAME_CLIENT_A);
    assert!(!harness.state.areas().is_available("client.read", 1));
    let lease = lease(&harness, AmmlInputBlocks(0));

    harness.state.input().turn(lease, 10.0, 0.0).unwrap();
    ClientGame::frame();

    assert_eq!(LookMoveGame::calls(), ["turn this=0x5000 x=0 y=10"]);
}
