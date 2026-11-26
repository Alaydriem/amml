use std::ffi::c_void;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use amml::{InputAdapter, ModListener, SessionAdapter};
use amml_abi::{
    AmmlBand, AmmlEventContext, AmmlEventHeader, AmmlInputBlocks, AmmlLeaseMode, AmmlModifiers, AmmlStatus, AmmlThread,
    KeyInput, WindowFocus,
};
use amml_data::Availability;
use amml_runtime::{LeaseTerms, ResourceId};

use windows::Win32::UI::WindowsAndMessaging::WM_KILLFOCUS;

use crate::support::{Harness, InputGame, LogFile, SessionGame, TestWindow};

const W: u16 = 0x57;
const ESCAPE: u16 = 0x1B;
const ALT: u16 = 0x12;
const GRAVE: u16 = 0xC0;

// What the `input.key` listener saw, one line per event.
static KEYS_SEEN: Mutex<Vec<String>> = Mutex::new(Vec::new());
// When set, the listener cancels every key down.
static CANCEL_DOWNS: AtomicBool = AtomicBool::new(false);

unsafe extern "C" fn on_key(
    event: *mut AmmlEventHeader,
    context: *const AmmlEventContext,
    _: *mut c_void,
) -> AmmlStatus {
    // SAFETY: the loader raises `input.key` with a `KeyInput` and a live context.
    let (key, context) = unsafe { (&mut *event.cast::<KeyInput>(), &*context) };
    KEYS_SEEN.lock().unwrap().push(format!(
        "key={:#x} down={} repeat={} cancelled={} injected={} thread={}",
        key.key, key.down, key.repeat, key.header.cancelled, key.header.injected, context.thread.0
    ));
    if key.down == 1 && CANCEL_DOWNS.load(Ordering::SeqCst) {
        key.header.cancelled = 1;
    }
    AmmlStatus::OK
}

// When set, the `window.focus` listener holds every focus loss back.
static HOLD_FOCUS: AtomicBool = AtomicBool::new(false);

unsafe extern "C" fn on_focus(event: *mut AmmlEventHeader, _: *const AmmlEventContext, _: *mut c_void) -> AmmlStatus {
    // SAFETY: the loader raises `window.focus` with a `WindowFocus`.
    let focus = unsafe { &mut *event.cast::<WindowFocus>() };
    if focus.gained == 0 && HOLD_FOCUS.load(Ordering::SeqCst) {
        focus.header.cancelled = 1;
    }
    AmmlStatus::OK
}

fn keys_seen() -> Vec<String> {
    KEYS_SEEN.lock().unwrap().clone()
}

// Installs the session and input adapters over `entries`; gives the harness and the input rows.
fn install(entries: &[(&'static str, amml_data::Resolved)]) -> (Harness, Vec<Availability>) {
    let harness = Harness::start_with_entries(entries);
    SessionAdapter::install(harness.state);
    let rows = InputAdapter::install(harness.state);
    harness
        .state
        .events()
        .listen("input.key", 1, AmmlBand::NORMAL, ModListener { owner: 0, callback: on_key, user: 0 })
        .ok();
    (harness, rows)
}

// `install` over the HID entries, after one frame (the gate is open, this thread is the game thread).
fn start_hid() -> Harness {
    let (harness, _) = install(&InputGame::hid_entries());
    SessionGame::frame();
    harness
}

// `install` over the `_feedKeyState` entry, after one frame.
fn start_processor() -> Harness {
    let (harness, _) = install(&InputGame::processor_entries());
    SessionGame::frame();
    harness
}

fn lease(harness: &Harness, blocks: AmmlInputBlocks, passthrough: Vec<u16>) -> ResourceId {
    let terms = LeaseTerms { mode: AmmlLeaseMode::SHARED, blocks, passthrough, preempt: false };
    harness.state.input().acquire(0, "@test/mod0", terms).unwrap()
}

fn shape_of(rows: &[Availability], id: &str) -> Option<&'static str> {
    rows.iter().find(|row| row.id == id).and_then(|row| row.shape)
}

#[test]
fn the_hid_shape_is_picked_when_its_entries_resolve() {
    let (harness, rows) = install(&InputGame::hid_entries());

    assert_eq!(shape_of(&rows, "input.key"), Some("hid"));
    assert_eq!(shape_of(&rows, "input.key_inject"), Some("hid"));
    assert!(harness.state.areas().is_available("input.key_inject", 1));
    assert!(harness.state.events().is_available("input.key", 1));
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "input feed bound");
    assert!(log.contains("area adapter") && log.contains(r#""area":"input.key_inject""#), "{log}");
    assert!(log.contains(r#""shape":"hid""#), "{log}");
}

#[test]
fn the_processor_shape_is_picked_on_70x() {
    let (harness, rows) = install(&InputGame::processor_entries());

    assert_eq!(shape_of(&rows, "input.key"), Some("key_processor"));
    assert_eq!(shape_of(&rows, "input.key_inject"), Some("key_processor"));
    assert!(harness.state.areas().is_available("input.key_inject", 1));
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "input feed bound");
    assert!(log.contains("area adapter") && log.contains(r#""shape":"key_processor""#), "{log}");
}

#[test]
fn no_key_shape_leaves_the_key_rows_not_available() {
    let (harness, rows) = install(&SessionGame::entries(&[]));

    assert_eq!(shape_of(&rows, "input.key_inject"), None);
    assert!(!harness.state.areas().is_available("input.key_inject", 1));
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "area not available");
    assert!(log.contains("HIDControllerGameCoreDesktop.onKeyDown"), "{log}");
}

#[test]
fn a_real_key_reaches_listeners_and_the_game() {
    let _harness = start_hid();

    InputGame::key_down(i32::from(W), 1);
    InputGame::key_down(i32::from(W), 1);
    InputGame::key_up(i32::from(W));

    assert_eq!(
        keys_seen(),
        [
            "key=0x57 down=1 repeat=0 cancelled=0 injected=0 thread=0",
            "key=0x57 down=1 repeat=1 cancelled=0 injected=0 thread=0",
            "key=0x57 down=0 repeat=0 cancelled=0 injected=0 thread=0",
        ]
    );
    assert_eq!(
        InputGame::calls(),
        ["down this=0x7c0000 key=0x57 origin=1", "down this=0x7c0000 key=0x57 origin=1", "up this=0x7c0000 key=0x57"]
    );
}

#[test]
fn a_cancelled_key_down_and_its_up_never_reach_the_game() {
    let _harness = start_hid();
    CANCEL_DOWNS.store(true, Ordering::SeqCst);

    InputGame::key_down(i32::from(W), 1);
    InputGame::key_up(i32::from(W));

    assert_eq!(
        keys_seen(),
        [
            "key=0x57 down=1 repeat=0 cancelled=0 injected=0 thread=0",
            "key=0x57 down=0 repeat=0 cancelled=0 injected=0 thread=0",
        ]
    );
    assert!(InputGame::calls().is_empty());
}

#[test]
fn a_cancelled_key_down_on_70x_never_reaches_the_game() {
    let _harness = start_processor();
    CANCEL_DOWNS.store(true, Ordering::SeqCst);

    InputGame::feed(0x1234, i32::from(W), 1);
    InputGame::feed(0x1234, i32::from(W), 0);

    assert_eq!(keys_seen().len(), 2);
    assert!(InputGame::calls().is_empty());
}

#[test]
fn an_up_of_a_key_that_is_not_down_passes_without_an_event() {
    let _harness = start_processor();

    for key in 0..=255 {
        InputGame::feed(0x1234, key, 0);
    }

    assert_eq!(InputGame::calls().len(), 256);
    assert_eq!(InputGame::calls()[0x57], "state this=0x1234 key=0x57 state=0");
    assert!(keys_seen().is_empty());
}

#[test]
fn a_blocking_lease_cancels_real_keys_before_listeners_except_passthrough() {
    let harness = start_hid();
    lease(&harness, AmmlInputBlocks::FULL, vec![ESCAPE]);

    InputGame::key_down(i32::from(W), 1);
    InputGame::key_up(i32::from(W));
    InputGame::key_down(i32::from(ESCAPE), 1);

    assert_eq!(
        keys_seen(),
        [
            "key=0x57 down=1 repeat=0 cancelled=1 injected=0 thread=0",
            "key=0x57 down=0 repeat=0 cancelled=1 injected=0 thread=0",
            "key=0x1b down=1 repeat=0 cancelled=0 injected=0 thread=0",
        ]
    );
    assert_eq!(InputGame::calls(), ["down this=0x7c0000 key=0x1b origin=1"]);
}

#[test]
fn an_injected_key_reaches_the_game_once_in_the_next_frame() {
    let harness = start_hid();
    let input = harness.state.input();
    let lease = lease(&harness, AmmlInputBlocks(0), Vec::new());
    assert_eq!(input.key_down(lease, W), Err(AmmlStatus::NOT_READY));
    InputGame::update();
    InputGame::clear();

    let hold = input.key_down(lease, W).unwrap();
    assert!(InputGame::calls().is_empty());
    SessionGame::frame();
    assert_eq!(InputGame::calls(), ["down this=0x7c0000 key=0x57 origin=1"]);
    assert_eq!(keys_seen(), ["key=0x57 down=1 repeat=0 cancelled=0 injected=1 thread=0"]);
    SessionGame::frame();
    input.release_hold(hold).unwrap();
    SessionGame::frame();

    assert_eq!(InputGame::calls(), ["down this=0x7c0000 key=0x57 origin=1", "up this=0x7c0000 key=0x57"]);
}

#[test]
fn an_injected_key_takes_the_origin_of_the_last_real_key_down() {
    let harness = start_hid();
    let input = harness.state.input();
    InputGame::key_down(0x41, 0);
    InputGame::key_up(0x41);
    InputGame::clear();
    let lease = lease(&harness, AmmlInputBlocks(0), Vec::new());

    input.key_down(lease, W).unwrap();
    SessionGame::frame();

    assert_eq!(InputGame::calls(), ["down this=0x7c0000 key=0x57 origin=0"]);
}

#[test]
fn the_processor_feed_injects_with_a_null_this() {
    let harness = start_processor();
    InputGame::feed(0x1234, 0x41, 1);
    InputGame::feed(0x1234, 0x41, 0);
    InputGame::clear();
    let lease = lease(&harness, AmmlInputBlocks(0), Vec::new());

    harness.state.input().key_down(lease, W).unwrap();
    SessionGame::frame();

    assert_eq!(InputGame::calls(), ["state this=0x0 key=0x57 state=1"]);
}

#[test]
fn injection_waits_for_the_first_real_feed_on_70x() {
    let harness = start_processor();
    let input = harness.state.input();
    let lease = lease(&harness, AmmlInputBlocks(0), Vec::new());
    assert_eq!(input.key_down(lease, W), Err(AmmlStatus::NOT_READY));

    // An up from a reset loop does not tell which thread feeds keys.
    InputGame::feed(0x1234, 0x41, 0);
    assert_eq!(input.key_down(lease, W), Err(AmmlStatus::NOT_READY));
    InputGame::feed(0x1234, 0x41, 1);

    assert!(input.key_down(lease, W).is_ok());
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "key feed thread is the game thread");
    assert!(log.contains("key feed thread is the game thread"), "{log}");
}

#[test]
fn a_feed_on_another_thread_makes_injection_not_available() {
    let harness = start_processor();
    let input = harness.state.input();
    let lease = lease(&harness, AmmlInputBlocks(0), Vec::new());

    // The focus reset loop on another thread sends an up for every key: no decision.
    std::thread::spawn(|| {
        for key in 0..=255 {
            InputGame::feed(0, key, 0);
        }
    })
    .join()
    .unwrap();
    assert_eq!(input.key_down(lease, W), Err(AmmlStatus::NOT_READY));
    assert!(harness.state.areas().is_available("input.key_inject", 1));

    std::thread::spawn(|| InputGame::feed(0, 0x41, 1)).join().unwrap();
    let real_calls = InputGame::calls().len();
    assert!(InputGame::threads().iter().all(|thread| *thread != std::thread::current().id()));
    assert_eq!(input.key_down(lease, W), Err(AmmlStatus::NOT_AVAILABLE));
    SessionGame::frame();

    assert_eq!(InputGame::calls().len(), real_calls);
    assert_eq!(real_calls, 257);
    assert_eq!(
        keys_seen(),
        [format!("key=0x41 down=1 repeat=0 cancelled=0 injected=0 thread={}", AmmlThread::INPUT.0)]
    );
    assert!(!harness.state.areas().is_available("input.key_inject", 1));
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "area withdrawn");
    assert_eq!(log.matches("key injection not available: the key feed is not the game thread").count(), 1, "{log}");
    assert!(log.contains("area withdrawn"), "{log}");
}

#[test]
fn a_real_reset_up_after_an_injected_hold_reaches_the_game_once() {
    let harness = start_processor();
    let input = harness.state.input();
    InputGame::feed(0x1234, 0x41, 1);
    InputGame::feed(0x1234, 0x41, 0);
    let lease = lease(&harness, AmmlInputBlocks(0), Vec::new());
    let hold = input.key_down(lease, W).unwrap();
    SessionGame::frame();
    InputGame::clear();

    InputGame::feed(0x1234, i32::from(W), 0);
    input.release_hold(hold).unwrap();
    SessionGame::frame();

    assert_eq!(InputGame::calls(), ["state this=0x1234 key=0x57 state=0"]);
    assert_eq!(keys_seen().len(), 3);
}

#[test]
fn a_blocked_real_press_of_an_injected_key_leaves_exactly_one_up() {
    let harness = start_hid();
    let input = harness.state.input();
    InputGame::update();
    InputGame::clear();
    let lease = lease(&harness, AmmlInputBlocks::KEYS, Vec::new());
    let hold = input.key_down(lease, W).unwrap();
    SessionGame::frame();

    InputGame::key_down(i32::from(W), 1);
    input.release_hold(hold).unwrap();
    SessionGame::frame();
    InputGame::key_up(i32::from(W));

    assert_eq!(InputGame::calls(), ["down this=0x7c0000 key=0x57 origin=1", "up this=0x7c0000 key=0x57"]);
}

#[test]
fn an_injected_key_during_a_blocked_real_press_reaches_the_game() {
    let harness = start_hid();
    let input = harness.state.input();
    InputGame::update();
    InputGame::clear();
    let lease = lease(&harness, AmmlInputBlocks::KEYS, Vec::new());

    InputGame::key_down(i32::from(W), 1);
    let hold = input.key_down(lease, W).unwrap();
    SessionGame::frame();
    InputGame::key_up(i32::from(W));
    input.release_hold(hold).unwrap();
    SessionGame::frame();

    assert_eq!(InputGame::calls(), ["down this=0x7c0000 key=0x57 origin=1", "up this=0x7c0000 key=0x57"]);
}

// The window events offered, a `window.focus` listener, and a test window under the loader's subclass.
fn attach_window(harness: &Harness) -> TestWindow {
    let window = harness.state.window();
    window.offer();
    let listener = ModListener { owner: 0, callback: on_focus, user: 0 };
    harness.state.events().listen("window.focus", 1, AmmlBand::NORMAL, listener).unwrap();
    let test = TestWindow::new(200, 100);
    assert!(window.attach(test.address()));
    test
}

// `WM_NULL`: a message the game window's procedure passes on with no event.
const WM_NULL: u32 = 0;

#[test]
fn on_70x_the_window_procedure_on_the_game_thread_makes_injection_ready_before_any_real_key() {
    let harness = start_processor();
    let input = harness.state.input();
    let lease = lease(&harness, AmmlInputBlocks(0), Vec::new());
    let window = attach_window(&harness);
    assert_eq!(input.key_down(lease, W), Err(AmmlStatus::NOT_READY));

    window.send(WM_NULL, 0, 0);

    assert!(input.key_down(lease, W).is_ok());
    SessionGame::frame();
    assert_eq!(InputGame::calls(), ["state this=0x0 key=0x57 state=1"]);
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "key feed thread is the game thread");
    assert!(log.contains(r#""from":"window""#), "{log}");
}

#[test]
fn a_real_key_down_off_the_game_thread_after_the_window_rule_withdraws_injection() {
    let harness = start_processor();
    let input = harness.state.input();
    let lease = lease(&harness, AmmlInputBlocks(0), Vec::new());
    let window = attach_window(&harness);
    window.send(WM_NULL, 0, 0);
    assert!(input.key_down(lease, W).is_ok());

    std::thread::spawn(|| InputGame::feed(0, 0x41, 1)).join().unwrap();

    assert_eq!(input.key_down(lease, W), Err(AmmlStatus::NOT_AVAILABLE));
    assert!(!harness.state.areas().is_available("input.key_inject", 1));
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "first real key down");
    assert!(log.contains(r#""in_window_proc":false"#) && log.contains(r#""game_thread":false"#), "{log}");
}

#[test]
fn the_key_reset_with_alt_held_raises_the_alt_up_and_clears_the_table() {
    let harness = start_hid();
    InputGame::key_down(i32::from(ALT), 1);
    InputGame::clear();

    InputGame::reset_keys();

    assert_eq!(
        keys_seen(),
        [
            "key=0x12 down=1 repeat=0 cancelled=0 injected=0 thread=0",
            "key=0x12 down=0 repeat=0 cancelled=0 injected=0 thread=0",
        ]
    );
    let keys = harness.state.input().keys();
    assert!(keys.real_down_keys().is_empty());
    assert_eq!(keys.modifiers(), AmmlModifiers(0));
    assert_eq!(InputGame::calls(), ["reset this=0x7c0000"]);
}

#[test]
fn a_key_after_the_reset_carries_no_stale_modifier() {
    let harness = start_hid();
    InputGame::key_down(i32::from(ALT), 1);

    InputGame::reset_keys();
    InputGame::key_down(i32::from(GRAVE), 1);

    assert_eq!(keys_seen().last().unwrap(), "key=0xc0 down=1 repeat=0 cancelled=0 injected=0 thread=0");
    assert_eq!(harness.state.input().keys().modifiers(), AmmlModifiers(0));
}

#[test]
fn the_key_reset_with_nothing_held_raises_nothing_and_reaches_the_game() {
    let _harness = start_hid();

    InputGame::reset_keys();

    assert!(keys_seen().is_empty());
    assert_eq!(InputGame::calls(), ["reset this=0x7c0000"]);
}

#[test]
fn the_key_reset_always_reaches_the_game_once_also_under_a_blocking_lease() {
    let harness = start_hid();
    lease(&harness, AmmlInputBlocks::FULL, Vec::new());
    InputGame::key_down(i32::from(W), 1);
    InputGame::key_down(i32::from(ALT), 1);

    InputGame::reset_keys();

    assert_eq!(InputGame::calls(), ["reset this=0x7c0000"]);
    assert!(harness.state.input().keys().real_down_keys().is_empty());
}

#[test]
fn the_key_reset_hook_is_logged_when_it_is_installed() {
    let (harness, _) = install(&InputGame::hid_entries());

    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "key reset hook installed");
    assert!(log.contains(r#""entry":"HIDController.resetKeys""#), "{log}");
}

#[test]
fn without_the_key_reset_entry_the_hid_shape_still_binds_and_logs_it() {
    let (harness, rows) = install(&InputGame::hid_entries_without_reset());
    SessionGame::frame();

    assert_eq!(shape_of(&rows, "input.key"), Some("hid"));
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "key reset hook not installed");
    assert!(log.contains(r#""entry":"HIDController.resetKeys""#), "{log}");
    InputGame::key_down(i32::from(W), 1);
    assert_eq!(keys_seen(), ["key=0x57 down=1 repeat=0 cancelled=0 injected=0 thread=0"]);
}

#[test]
fn a_focus_loss_releases_the_held_real_keys_without_sending_ups_to_the_game() {
    let harness = start_hid();
    let window = attach_window(&harness);
    InputGame::key_down(i32::from(ALT), 1);
    InputGame::clear();

    window.send(WM_KILLFOCUS, 0, 0);

    assert_eq!(keys_seen().last().unwrap(), "key=0x12 down=0 repeat=0 cancelled=0 injected=0 thread=0");
    assert!(harness.state.input().keys().real_down_keys().is_empty());
    assert!(InputGame::calls().is_empty());
}

#[test]
fn a_focus_loss_a_listener_holds_back_keeps_the_held_real_keys() {
    let harness = start_hid();
    let window = attach_window(&harness);
    HOLD_FOCUS.store(true, Ordering::SeqCst);
    InputGame::key_down(i32::from(ALT), 1);

    window.send(WM_KILLFOCUS, 0, 0);

    assert_eq!(keys_seen(), ["key=0x12 down=1 repeat=0 cancelled=0 injected=0 thread=0"]);
    assert_eq!(harness.state.input().keys().real_down_keys(), vec![ALT]);
}

#[test]
fn a_focus_loss_then_the_hid_reset_raises_one_up() {
    let harness = start_hid();
    let window = attach_window(&harness);
    InputGame::key_down(i32::from(ALT), 1);

    window.send(WM_KILLFOCUS, 0, 0);
    InputGame::reset_keys();

    assert_eq!(keys_seen().iter().filter(|line| line.contains("down=0")).count(), 1);
}

#[test]
fn a_focus_loss_then_the_70x_reset_loop_raises_one_up_and_every_up_reaches_the_game() {
    let harness = start_processor();
    let window = attach_window(&harness);
    InputGame::feed(0x1234, i32::from(ALT), 1);
    InputGame::clear();

    window.send(WM_KILLFOCUS, 0, 0);
    for key in 0..=255 {
        InputGame::feed(0x1234, key, 0);
    }

    assert_eq!(
        keys_seen(),
        [
            "key=0x12 down=1 repeat=0 cancelled=0 injected=0 thread=0",
            "key=0x12 down=0 repeat=0 cancelled=0 injected=0 thread=0",
        ]
    );
    assert_eq!(InputGame::calls().len(), 256);
    assert!(harness.state.input().keys().real_down_keys().is_empty());
}
