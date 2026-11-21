use std::ffi::c_void;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use amml::{EightArgMouseFeed, InputAdapter, ModListener, SessionAdapter, SevenArgMouseFeed};
use amml_abi::{
    AmmlBand, AmmlEventContext, AmmlEventHeader, AmmlInputBlocks, AmmlLeaseMode, AmmlMouseButton, AmmlStatus,
    AmmlThread, MouseInput,
};
use amml_data::Availability;
use amml_runtime::{LeaseTerms, ResourceId};

use crate::support::{Harness, InputGame, LogFile, SessionGame};

const MOVE: u8 = 0;
const LEFT: u8 = 1;
const RIGHT: u8 = 2;
const WHEEL: u8 = 4;
const X2: u8 = 6;

// What the `input.mouse` listener saw, one line per event.
static MOUSE_SEEN: Mutex<Vec<String>> = Mutex::new(Vec::new());
// When set, the listener cancels every button down.
static CANCEL_DOWNS: AtomicBool = AtomicBool::new(false);
// When set, the listener moves the pointer to 10, 20.
static MOVE_POINTER: AtomicBool = AtomicBool::new(false);

unsafe extern "C" fn on_mouse(
    event: *mut AmmlEventHeader,
    context: *const AmmlEventContext,
    _: *mut c_void,
) -> AmmlStatus {
    // SAFETY: the loader raises `input.mouse` with a `MouseInput` and a live context.
    let (mouse, context) = unsafe { (&mut *event.cast::<MouseInput>(), &*context) };
    MOUSE_SEEN.lock().unwrap().push(format!(
        "action={} button={} down={} x={} y={} dx={} dy={} notches={} cancelled={} injected={} thread={}",
        mouse.action.0,
        mouse.button.0,
        mouse.down,
        mouse.x,
        mouse.y,
        mouse.dx,
        mouse.dy,
        mouse.notches,
        mouse.header.cancelled,
        mouse.header.injected,
        context.thread.0
    ));
    if mouse.down == 1 && CANCEL_DOWNS.load(Ordering::SeqCst) {
        mouse.header.cancelled = 1;
    }
    if MOVE_POINTER.load(Ordering::SeqCst) {
        mouse.x = 10;
        mouse.y = 20;
    }
    AmmlStatus::OK
}

fn seen() -> Vec<String> {
    MOUSE_SEEN.lock().unwrap().clone()
}

fn device() -> usize {
    InputGame::mouse_device()
}

// Installs the session and input adapters over `entries`; gives the harness and the input rows.
fn install(entries: &[(&'static str, amml_data::Resolved)]) -> (Harness, Vec<Availability>) {
    let harness = Harness::start_with_entries(entries);
    SessionAdapter::install(harness.state);
    let rows = InputAdapter::install(harness.state);
    harness
        .state
        .events()
        .listen("input.mouse", 1, AmmlBand::NORMAL, ModListener { owner: 0, callback: on_mouse, user: 0 })
        .ok();
    (harness, rows)
}

// `install` over the seven-argument entries, after one frame (the gate is open, this thread is the game
// thread).
fn start_seven() -> Harness {
    let (harness, _) = install(&InputGame::seven_arg_mouse_entries());
    SessionGame::frame();
    harness
}

// `install` over the eight-argument entries, after one frame.
fn start_eight() -> Harness {
    let (harness, _) = install(&InputGame::eight_arg_mouse_entries());
    SessionGame::frame();
    harness
}

fn lease(harness: &Harness, blocks: AmmlInputBlocks) -> ResourceId {
    let terms = LeaseTerms { mode: AmmlLeaseMode::SHARED, blocks, passthrough: Vec::new(), preempt: false };
    harness.state.input().acquire(0, "@test/mod0", terms).unwrap()
}

fn shape_of(rows: &[Availability], id: &str) -> Option<&'static str> {
    rows.iter().find(|row| row.id == id).and_then(|row| row.shape)
}

#[test]
fn the_seven_arg_shape_is_picked_on_26x() {
    let (harness, rows) = install(&InputGame::seven_arg_mouse_entries());

    assert_eq!(SevenArgMouseFeed::FEED_ENTRY, "MouseDevice.feed");
    assert_eq!(shape_of(&rows, "input.mouse"), Some("seven_arg_feed"));
    assert_eq!(shape_of(&rows, "input.mouse_inject"), Some("seven_arg_feed"));
    assert!(harness.state.areas().is_available("input.mouse_inject", 1));
    assert!(harness.state.events().is_available("input.mouse", 1));
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "input feed bound");
    assert!(log.contains(r#""area":"input.mouse_inject""#) && log.contains(r#""shape":"seven_arg_feed""#), "{log}");
}

#[test]
fn the_eight_arg_shape_is_picked_on_70x() {
    let (harness, rows) = install(&InputGame::eight_arg_mouse_entries());

    assert_eq!(EightArgMouseFeed::FEED_ENTRY, "MouseDevice.feedWithControllerId");
    assert_eq!(shape_of(&rows, "input.mouse"), Some("eight_arg_feed"));
    assert_eq!(shape_of(&rows, "input.mouse_inject"), Some("eight_arg_feed"));
    assert!(harness.state.areas().is_available("input.mouse_inject", 1));
}

#[test]
fn a_listener_can_move_the_pointer() {
    let _harness = start_seven();
    MOVE_POINTER.store(true, Ordering::SeqCst);

    InputGame::mouse(device(), MOVE, 0, 1, 2, 3, 4);

    assert_eq!(seen(), ["action=0 button=0 down=0 x=1 y=2 dx=3 dy=4 notches=0 cancelled=0 injected=0 thread=0"]);
    assert_eq!(InputGame::calls(), ["mouse this=device action=0 data=0 x=10 y=20 dx=3 dy=4 motionless=0"]);
}

#[test]
fn a_real_button_and_wheel_reach_listeners_and_the_game() {
    let _harness = start_seven();

    InputGame::mouse(device(), X2, 1, 5, 6, 0, 0);
    InputGame::mouse(device(), X2, 0, 5, 6, 0, 0);
    InputGame::mouse(device(), WHEEL, -120, 5, 6, 0, 0);

    assert_eq!(
        seen(),
        [
            "action=1 button=4 down=1 x=5 y=6 dx=0 dy=0 notches=0 cancelled=0 injected=0 thread=0",
            "action=1 button=4 down=0 x=5 y=6 dx=0 dy=0 notches=0 cancelled=0 injected=0 thread=0",
            "action=2 button=0 down=0 x=5 y=6 dx=0 dy=0 notches=-1 cancelled=0 injected=0 thread=0",
        ]
    );
    assert_eq!(
        InputGame::calls(),
        [
            "mouse this=device action=6 data=1 x=5 y=6 dx=0 dy=0 motionless=0",
            "mouse this=device action=6 data=0 x=5 y=6 dx=0 dy=0 motionless=0",
            "mouse this=device action=4 data=-120 x=5 y=6 dx=0 dy=0 motionless=0",
        ]
    );
}

#[test]
fn a_cancelled_button_down_cancels_its_up() {
    let _harness = start_seven();
    CANCEL_DOWNS.store(true, Ordering::SeqCst);

    InputGame::mouse(device(), LEFT, 1, 5, 6, 0, 0);
    InputGame::mouse(device(), LEFT, 0, 5, 6, 0, 0);

    assert_eq!(
        seen(),
        [
            "action=1 button=0 down=1 x=5 y=6 dx=0 dy=0 notches=0 cancelled=0 injected=0 thread=0",
            "action=1 button=0 down=0 x=5 y=6 dx=0 dy=0 notches=0 cancelled=1 injected=0 thread=0",
        ]
    );
    assert!(InputGame::calls().is_empty());
}

#[test]
fn a_feed_down_whose_window_down_was_cancelled_is_raised_cancelled_and_held_back() {
    let harness = start_seven();
    harness.state.window().pairs().window_button(AmmlMouseButton::LEFT, true, true);

    InputGame::mouse(device(), LEFT, 1, 5, 6, 0, 0);
    InputGame::mouse(device(), LEFT, 0, 5, 6, 0, 0);
    InputGame::mouse(device(), RIGHT, 1, 5, 6, 0, 0);

    assert_eq!(
        seen(),
        [
            "action=1 button=0 down=1 x=5 y=6 dx=0 dy=0 notches=0 cancelled=1 injected=0 thread=0",
            "action=1 button=0 down=0 x=5 y=6 dx=0 dy=0 notches=0 cancelled=1 injected=0 thread=0",
            "action=1 button=1 down=1 x=5 y=6 dx=0 dy=0 notches=0 cancelled=0 injected=0 thread=0",
        ]
    );
    assert_eq!(InputGame::calls(), ["mouse this=device action=2 data=1 x=5 y=6 dx=0 dy=0 motionless=0"]);
}

#[test]
fn an_up_whose_down_the_loader_never_saw_reaches_the_game_without_an_event() {
    let _harness = start_seven();

    InputGame::mouse(device(), RIGHT, 0, 5, 6, 0, 0);

    assert!(seen().is_empty());
    assert_eq!(InputGame::calls(), ["mouse this=device action=2 data=0 x=5 y=6 dx=0 dy=0 motionless=0"]);
}

#[test]
fn a_feed_on_another_device_passes_without_an_event() {
    let harness = start_seven();
    lease(&harness, AmmlInputBlocks::MOUSE_BUTTONS);

    InputGame::mouse(0x5000, LEFT, 1, 5, 6, 0, 0);

    assert!(seen().is_empty());
    assert_eq!(InputGame::calls(), ["mouse this=0x5000 action=1 data=1 x=5 y=6 dx=0 dy=0 motionless=0"]);
}

#[test]
fn a_mouse_buttons_lease_cancels_real_buttons_before_listeners() {
    let harness = start_seven();
    lease(&harness, AmmlInputBlocks::MOUSE_BUTTONS);

    InputGame::mouse(device(), LEFT, 1, 5, 6, 0, 0);
    InputGame::mouse(device(), LEFT, 0, 5, 6, 0, 0);

    assert_eq!(
        seen(),
        [
            "action=1 button=0 down=1 x=5 y=6 dx=0 dy=0 notches=0 cancelled=1 injected=0 thread=0",
            "action=1 button=0 down=0 x=5 y=6 dx=0 dy=0 notches=0 cancelled=1 injected=0 thread=0",
        ]
    );
    assert!(InputGame::calls().is_empty());
}

#[test]
fn moves_and_the_wheel_pass_a_mouse_buttons_lease() {
    let harness = start_seven();
    lease(&harness, AmmlInputBlocks::FULL);

    InputGame::mouse(device(), MOVE, 0, 5, 6, 1, -1);
    InputGame::mouse(device(), WHEEL, 120, 5, 6, 0, 0);

    assert_eq!(
        seen(),
        [
            "action=0 button=0 down=0 x=5 y=6 dx=1 dy=-1 notches=0 cancelled=0 injected=0 thread=0",
            "action=2 button=0 down=0 x=5 y=6 dx=0 dy=0 notches=1 cancelled=0 injected=0 thread=0",
        ]
    );
    assert_eq!(
        InputGame::calls(),
        [
            "mouse this=device action=0 data=0 x=5 y=6 dx=1 dy=-1 motionless=0",
            "mouse this=device action=4 data=120 x=5 y=6 dx=0 dy=0 motionless=0",
        ]
    );
}

#[test]
fn the_up_of_a_button_held_before_a_blocking_lease_reaches_the_game() {
    let harness = start_seven();
    InputGame::mouse(device(), LEFT, 1, 5, 6, 0, 0);
    lease(&harness, AmmlInputBlocks::MOUSE_BUTTONS);

    InputGame::mouse(device(), LEFT, 0, 5, 6, 0, 0);

    assert_eq!(seen()[1], "action=1 button=0 down=0 x=5 y=6 dx=0 dy=0 notches=0 cancelled=1 injected=0 thread=0");
    assert_eq!(
        InputGame::calls(),
        [
            "mouse this=device action=1 data=1 x=5 y=6 dx=0 dy=0 motionless=0",
            "mouse this=device action=1 data=0 x=5 y=6 dx=0 dy=0 motionless=0",
        ]
    );
}

#[test]
fn an_injected_click_reaches_the_game_in_the_next_frame() {
    let harness = start_seven();
    let input = harness.state.input();
    let lease = lease(&harness, AmmlInputBlocks(0));
    assert_eq!(input.button_down(lease, AmmlMouseButton::LEFT), Err(AmmlStatus::NOT_READY));
    InputGame::mouse(device(), MOVE, 0, 30, 40, 0, 0);
    InputGame::clear();
    MOUSE_SEEN.lock().unwrap().clear();

    let hold = input.button_down(lease, AmmlMouseButton::LEFT).unwrap();
    assert!(InputGame::calls().is_empty());
    SessionGame::frame();
    assert_eq!(InputGame::calls(), ["mouse this=device action=1 data=1 x=30 y=40 dx=0 dy=0 motionless=0"]);
    input.release_hold(hold).unwrap();
    SessionGame::frame();

    assert_eq!(
        InputGame::calls(),
        [
            "mouse this=device action=1 data=1 x=30 y=40 dx=0 dy=0 motionless=0",
            "mouse this=device action=1 data=0 x=30 y=40 dx=0 dy=0 motionless=0",
        ]
    );
    assert_eq!(
        seen(),
        [
            "action=1 button=0 down=1 x=0 y=0 dx=0 dy=0 notches=0 cancelled=0 injected=1 thread=0",
            "action=1 button=0 down=0 x=0 y=0 dx=0 dy=0 notches=0 cancelled=0 injected=1 thread=0",
        ]
    );
}

#[test]
fn a_real_up_ends_an_injected_hold_of_that_button() {
    let harness = start_seven();
    let input = harness.state.input();
    InputGame::mouse(device(), MOVE, 0, 30, 40, 0, 0);
    let lease = lease(&harness, AmmlInputBlocks(0));
    let hold = input.button_down(lease, AmmlMouseButton::LEFT).unwrap();
    SessionGame::frame();
    InputGame::clear();

    InputGame::mouse(device(), LEFT, 1, 30, 40, 0, 0);
    InputGame::mouse(device(), LEFT, 0, 30, 40, 0, 0);
    input.release_hold(hold).unwrap();
    SessionGame::frame();

    assert_eq!(
        InputGame::calls(),
        [
            "mouse this=device action=1 data=1 x=30 y=40 dx=0 dy=0 motionless=0",
            "mouse this=device action=1 data=0 x=30 y=40 dx=0 dy=0 motionless=0",
        ]
    );
}

#[test]
fn one_wheel_notch_is_one_feed() {
    let harness = start_seven();
    let input = harness.state.input();
    InputGame::mouse(device(), MOVE, 0, 30, 40, 0, 0);
    InputGame::clear();
    MOUSE_SEEN.lock().unwrap().clear();
    let lease = lease(&harness, AmmlInputBlocks(0));

    input.wheel(lease, 2.0).unwrap();
    input.wheel(lease, -1.0).unwrap();
    SessionGame::frame();

    assert_eq!(
        InputGame::calls(),
        [
            "mouse this=device action=4 data=120 x=30 y=40 dx=0 dy=0 motionless=0",
            "mouse this=device action=4 data=120 x=30 y=40 dx=0 dy=0 motionless=0",
            "mouse this=device action=4 data=-120 x=30 y=40 dx=0 dy=0 motionless=0",
        ]
    );
    assert_eq!(
        seen(),
        [
            "action=2 button=0 down=0 x=0 y=0 dx=0 dy=0 notches=1 cancelled=0 injected=1 thread=0",
            "action=2 button=0 down=0 x=0 y=0 dx=0 dy=0 notches=1 cancelled=0 injected=1 thread=0",
            "action=2 button=0 down=0 x=0 y=0 dx=0 dy=0 notches=-1 cancelled=0 injected=1 thread=0",
        ]
    );
}

#[test]
fn two_half_notches_send_one_notch() {
    let harness = start_seven();
    let input = harness.state.input();
    InputGame::mouse(device(), MOVE, 0, 30, 40, 0, 0);
    InputGame::clear();
    let lease = lease(&harness, AmmlInputBlocks(0));

    input.wheel(lease, 0.5).unwrap();
    SessionGame::frame();
    assert!(InputGame::calls().is_empty());
    input.wheel(lease, 0.5).unwrap();
    SessionGame::frame();

    assert_eq!(InputGame::calls(), ["mouse this=device action=4 data=120 x=30 y=40 dx=0 dy=0 motionless=0"]);
}

#[test]
fn the_eight_arg_shape_passes_its_extra_argument_through() {
    let _harness = start_eight();

    InputGame::mouse_eight(device(), LEFT, 1, 5, 6, 0, 0, 7);
    InputGame::mouse_eight(device(), LEFT, 0, 5, 6, 0, 0, -3);

    assert_eq!(
        InputGame::calls(),
        [
            "mouse this=device action=1 data=1 x=5 y=6 dx=0 dy=0 controller=7 motionless=1",
            "mouse this=device action=1 data=0 x=5 y=6 dx=0 dy=0 controller=-3 motionless=1",
        ]
    );
    assert_eq!(seen().len(), 2);
}

#[test]
fn the_eight_arg_shape_injects_with_controller_zero() {
    let harness = start_eight();
    let input = harness.state.input();
    InputGame::mouse_eight(device(), MOVE, 0, 30, 40, 0, 0, 7);
    InputGame::clear();
    let lease = lease(&harness, AmmlInputBlocks(0));

    input.button_down(lease, AmmlMouseButton::RIGHT).unwrap();
    input.wheel(lease, -1.0).unwrap();
    SessionGame::frame();

    assert_eq!(
        InputGame::calls(),
        [
            "mouse this=device action=2 data=1 x=30 y=40 dx=0 dy=0 controller=0 motionless=0",
            "mouse this=device action=4 data=-120 x=30 y=40 dx=0 dy=0 controller=0 motionless=0",
        ]
    );
}

#[test]
fn with_no_mouse_entry_mouse_injection_is_not_available() {
    let (harness, rows) = install(&SessionGame::entries(&[]));
    SessionGame::frame();
    let input = harness.state.input();
    let lease = lease(&harness, AmmlInputBlocks(0));

    assert_eq!(input.button_down(lease, AmmlMouseButton::LEFT), Err(AmmlStatus::NOT_AVAILABLE));
    assert_eq!(input.wheel(lease, 1.0), Err(AmmlStatus::NOT_AVAILABLE));
    SessionGame::frame();

    assert_eq!(shape_of(&rows, "input.mouse_inject"), None);
    assert!(!harness.state.areas().is_available("input.mouse_inject", 1));
    assert!(!harness.state.events().is_available("input.mouse", 1));
    assert!(InputGame::calls().is_empty());
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), r#""area":"input.mouse_inject""#);
    let line = r#""area":"input.mouse_inject","missing":"MouseDevice.feed, MouseDevice.instance""#;
    assert_eq!(log.matches(line).count(), 1, "{log}");
    assert!(log.contains("area not available"), "{log}");
}

#[test]
fn a_feed_on_another_thread_makes_mouse_injection_not_available() {
    let harness = start_seven();
    let input = harness.state.input();
    let lease = lease(&harness, AmmlInputBlocks(0));

    std::thread::spawn(|| InputGame::mouse(InputGame::mouse_device(), MOVE, 0, 5, 6, 0, 0)).join().unwrap();
    assert_eq!(input.button_down(lease, AmmlMouseButton::LEFT), Err(AmmlStatus::NOT_AVAILABLE));
    SessionGame::frame();

    assert_eq!(InputGame::calls().len(), 1);
    assert_eq!(
        seen(),
        [format!(
            "action=0 button=0 down=0 x=5 y=6 dx=0 dy=0 notches=0 cancelled=0 injected=0 thread={}",
            AmmlThread::INPUT.0
        )]
    );
    assert!(!harness.state.areas().is_available("input.mouse_inject", 1));
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "area withdrawn");
    assert_eq!(log.matches("mouse injection not available: the mouse feed is not the game thread").count(), 1, "{log}");
}

#[test]
fn a_raw_move_does_not_move_the_injection_position() {
    let harness = start_seven();
    let input = harness.state.input();
    InputGame::mouse(device(), MOVE, 0, 30, 40, 0, 0);
    InputGame::mouse(device(), MOVE, 0, 0, 0, 5, -5);
    InputGame::clear();
    let lease = lease(&harness, AmmlInputBlocks(0));

    input.button_down(lease, AmmlMouseButton::LEFT).unwrap();
    SessionGame::frame();

    assert_eq!(InputGame::calls(), ["mouse this=device action=1 data=1 x=30 y=40 dx=0 dy=0 motionless=0"]);
}
