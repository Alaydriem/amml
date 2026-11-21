use std::ffi::c_void;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicU8, AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use amml::{LoaderState, ModListener, ModLoader, ModPhase, SessionAdapter, SizeMessage, WindowProc};
use amml_abi::{
    AmmlApi, AmmlBand, AmmlCursorClip, AmmlCursorShape, AmmlEventContext, AmmlEventHeader, AmmlPointerKind,
    AmmlServices, AmmlSize, AmmlStatus, AmmlStr, AmmlThread, AmmlWindowTable, WindowChar, WindowCursor, WindowFocus,
    WindowPointer, WindowSize,
};
use amml_runtime::ModRegistry;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CallWindowProcW, GWLP_WNDPROC, GetWindowLongPtrW, HTCLIENT, SIZE_RESTORED, SetWindowLongPtrW, WM_CHAR, WM_CLOSE,
    WM_DESTROY, WM_ENDSESSION, WM_KILLFOCUS, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_NCDESTROY, WM_SETCURSOR,
    WM_SETFOCUS, WM_SIZE, WNDPROC,
};

use crate::support::{CLIENT_A, EchoMod, EventsMod, Harness, LogFile, MessageLog, Seen, SessionGame, TestWindow};

const WIDTH: u32 = 1000;
const HEIGHT: u32 = 500;

// One line per event a listener saw.
static SEEN: Mutex<Vec<String>> = Mutex::new(Vec::new());
// The listeners cancel button downs, button ups, and focus events.
static CANCEL_DOWNS: AtomicBool = AtomicBool::new(false);
static CANCEL_UPS: AtomicBool = AtomicBool::new(false);
static CANCEL_FOCUS: AtomicBool = AtomicBool::new(false);
// The size listener writes 800 x 600.
static WRITE_SIZE: AtomicBool = AtomicBool::new(false);
// The size listener asks for 1280 x 720 when it sees a size the loader did not post.
static REQUEST_FROM_LISTENER: AtomicBool = AtomicBool::new(false);
// The cursor listener writes this shape.
static SHAPE: AtomicU8 = AtomicU8::new(0);

fn seen() -> Vec<String> {
    SEEN.lock().unwrap().clone()
}

fn note(line: String) {
    SEEN.lock().unwrap().push(line);
}

unsafe extern "C" fn on_pointer(
    event: *mut AmmlEventHeader,
    context: *const AmmlEventContext,
    _: *mut c_void,
) -> AmmlStatus {
    // SAFETY: the loader raises `window.pointer` with a `WindowPointer` and a live context.
    let (pointer, context) = unsafe { (&mut *event.cast::<WindowPointer>(), &*context) };
    note(format!(
        "pointer kind={} button={} down={} client={},{} buffer={},{} cancelled={} thread={}",
        pointer.kind.0,
        pointer.button.0,
        pointer.down,
        pointer.client.x,
        pointer.client.y,
        pointer.buffer.x,
        pointer.buffer.y,
        pointer.header.cancelled,
        context.thread.0
    ));
    let cancel = match (pointer.kind, pointer.down) {
        (AmmlPointerKind::BUTTON, 1) => CANCEL_DOWNS.load(Ordering::SeqCst),
        (AmmlPointerKind::BUTTON, _) => CANCEL_UPS.load(Ordering::SeqCst),
        _ => false,
    };
    if cancel {
        pointer.header.cancelled = 1;
    }
    AmmlStatus::OK
}

unsafe extern "C" fn on_char(event: *mut AmmlEventHeader, _: *const AmmlEventContext, _: *mut c_void) -> AmmlStatus {
    // SAFETY: the loader raises `window.char` with a `WindowChar`.
    let typed = unsafe { &*event.cast::<WindowChar>() };
    note(format!("char {:#x}", typed.ch));
    AmmlStatus::OK
}

unsafe extern "C" fn on_focus(event: *mut AmmlEventHeader, _: *const AmmlEventContext, _: *mut c_void) -> AmmlStatus {
    // SAFETY: the loader raises `window.focus` with a `WindowFocus`.
    let focus = unsafe { &mut *event.cast::<WindowFocus>() };
    note(format!("focus gained={} replayed={}", focus.gained, focus.replayed));
    if CANCEL_FOCUS.load(Ordering::SeqCst) {
        focus.header.cancelled = 1;
    }
    AmmlStatus::OK
}

unsafe extern "C" fn on_size(event: *mut AmmlEventHeader, _: *const AmmlEventContext, _: *mut c_void) -> AmmlStatus {
    // SAFETY: the loader raises `window.size` with a `WindowSize`.
    let size = unsafe { &mut *event.cast::<WindowSize>() };
    note(format!(
        "size {}x{} injected={} origin={}",
        size.width, size.height, size.header.injected, size.header.origin.0
    ));
    if WRITE_SIZE.load(Ordering::SeqCst) {
        size.width = 800;
        size.height = 600;
    }
    if REQUEST_FROM_LISTENER.load(Ordering::SeqCst) && size.header.injected == 0 {
        let window = LoaderState::get().unwrap().window();
        window.request_size(0, Some(AmmlSize { width: 1280, height: 720 })).unwrap();
    }
    AmmlStatus::OK
}

unsafe extern "C" fn on_cursor(event: *mut AmmlEventHeader, _: *const AmmlEventContext, _: *mut c_void) -> AmmlStatus {
    // SAFETY: the loader raises `window.cursor` with a `WindowCursor`.
    let cursor = unsafe { &mut *event.cast::<WindowCursor>() };
    cursor.shape = AmmlCursorShape(SHAPE.load(Ordering::SeqCst));
    note("cursor".to_string());
    AmmlStatus::OK
}

// A loader with the window events offered and a listener on each, and a test window under the loader's
// subclass, with the recorder as the game's procedure.
fn start() -> (Harness, TestWindow) {
    let harness = Harness::start();
    let window = harness.state.window();
    window.offer();
    let listen = |event: &str, callback| {
        let listener = ModListener { owner: 0, callback, user: 0 };
        harness.state.events().listen(event, 1, AmmlBand::NORMAL, listener).unwrap();
    };
    listen("window.pointer", on_pointer);
    listen("window.char", on_char);
    listen("window.focus", on_focus);
    listen("window.size", on_size);
    listen("window.cursor", on_cursor);
    let test = TestWindow::new(WIDTH, HEIGHT);
    MessageLog::install(test.hwnd);
    assert!(window.attach(test.address()));
    MessageLog::clear();
    (harness, test)
}

fn size_seen() -> Vec<isize> {
    MessageLog::seen_of(WM_SIZE).into_iter().map(|seen| seen.lparam).collect()
}

fn window_table(harness: &Harness) -> &'static AmmlWindowTable {
    // SAFETY: the loader's tables live for the process.
    let services = unsafe { AmmlApi::services(harness.state.api()) }.unwrap();
    // SAFETY: as above; the services table is v3.
    let table = unsafe { AmmlServices::window(services) }.unwrap();
    // SAFETY: the window table is a static of the loader.
    unsafe { &*table }
}

#[test]
fn a_cancelled_click_never_reaches_the_window() {
    let (_harness, window) = start();
    CANCEL_DOWNS.store(true, Ordering::SeqCst);

    assert_eq!(window.send(WM_LBUTTONDOWN, 1, TestWindow::point(10, 20)), 0);
    window.send(WM_LBUTTONUP, 0, TestWindow::point(10, 20));

    assert!(MessageLog::seen_of(WM_LBUTTONDOWN).is_empty());
    assert!(MessageLog::seen_of(WM_LBUTTONUP).is_empty());
    let lines = seen();
    assert_eq!(lines.len(), 2, "{lines:?}");
    assert!(lines[1].contains("down=0") && lines[1].contains("cancelled=1"), "{lines:?}");
}

#[test]
fn an_up_whose_down_reached_the_game_cannot_be_cancelled() {
    let (_harness, window) = start();
    CANCEL_UPS.store(true, Ordering::SeqCst);

    window.send(WM_LBUTTONDOWN, 1, TestWindow::point(10, 20));
    window.send(WM_LBUTTONUP, 0, TestWindow::point(10, 20));

    assert_eq!(MessageLog::seen_of(WM_LBUTTONDOWN).len(), 1);
    assert_eq!(MessageLog::seen_of(WM_LBUTTONUP).len(), 1);
}

#[test]
fn pointer_events_carry_client_and_buffer_points() {
    let (harness, window) = start();
    harness.state.window().set_buffer(AmmlSize { width: 2000, height: 1000 });

    window.send(WM_MOUSEMOVE, 0, TestWindow::point(10, 10));

    let thread = AmmlThread::WINDOW.0;
    assert_eq!(
        seen(),
        vec![format!("pointer kind=0 button=0 down=0 client=10,10 buffer=20,20 cancelled=0 thread={thread}")]
    );
    assert_eq!(MessageLog::seen_of(WM_MOUSEMOVE).len(), 1);
}

#[test]
fn surrogate_pairs_arrive_as_one_char() {
    let (_harness, window) = start();

    window.send(WM_CHAR, 0xD83D, 1);
    window.send(WM_CHAR, 0xDE00, 1);

    assert_eq!(seen(), vec!["char 0x1f600".to_string()]);
    let units: Vec<usize> = MessageLog::seen_of(WM_CHAR).into_iter().map(|seen| seen.wparam).collect();
    assert_eq!(units, vec![0xD83D, 0xDE00]);
}

#[test]
fn a_held_focus_loss_is_replayed_once_with_replayed_set() {
    let (harness, window) = start();
    CANCEL_FOCUS.store(true, Ordering::SeqCst);

    assert_eq!(window.send(WM_KILLFOCUS, 0, 0), 0);
    assert!(MessageLog::seen_of(WM_KILLFOCUS).is_empty());

    harness.state.window().replay_focus_loss().unwrap();
    window.pump();
    harness.state.window().replay_focus_loss().unwrap();
    window.pump();

    assert_eq!(MessageLog::seen_of(WM_KILLFOCUS), vec![Seen { message: WM_KILLFOCUS, wparam: 0, lparam: 0 }]);
    assert_eq!(seen(), vec!["focus gained=0 replayed=0", "focus gained=0 replayed=1"]);
}

#[test]
fn a_gain_forgets_a_held_loss() {
    let (harness, window) = start();
    CANCEL_FOCUS.store(true, Ordering::SeqCst);

    window.send(WM_KILLFOCUS, 0, 0);
    window.send(WM_SETFOCUS, 0, 0);
    harness.state.window().replay_focus_loss().unwrap();
    window.pump();

    assert!(MessageLog::seen_of(WM_KILLFOCUS).is_empty());
    assert_eq!(MessageLog::seen_of(WM_SETFOCUS).len(), 1);
    assert_eq!(seen(), vec!["focus gained=0 replayed=0", "focus gained=1 replayed=0"]);
}

#[test]
fn a_written_window_size_reaches_the_game() {
    let (_harness, window) = start();
    WRITE_SIZE.store(true, Ordering::SeqCst);

    window.send(WM_SIZE, SIZE_RESTORED as usize, SizeMessage::size_param(WIDTH, HEIGHT));

    assert_eq!(size_seen(), vec![SizeMessage::size_param(800, 600)]);
    assert_eq!(seen(), vec![format!("size {WIDTH}x{HEIGHT} injected=0 origin=0")]);
}

#[test]
fn a_requested_size_raises_one_injected_window_size_and_no_loop() {
    let (_harness, window) = start();
    REQUEST_FROM_LISTENER.store(true, Ordering::SeqCst);

    window.send(WM_SIZE, SIZE_RESTORED as usize, SizeMessage::size_param(WIDTH, HEIGHT));
    window.pump();

    assert_eq!(
        seen(),
        vec![format!("size {WIDTH}x{HEIGHT} injected=0 origin=0"), "size 1280x720 injected=1 origin=1".to_string()]
    );
    assert_eq!(size_seen(), vec![SizeMessage::size_param(WIDTH, HEIGHT), SizeMessage::size_param(1280, 720)]);
}

#[test]
fn a_set_cursor_shape_answers_wm_setcursor() {
    let (_harness, window) = start();
    SHAPE.store(AmmlCursorShape::HAND.0, Ordering::SeqCst);

    let result = window.send(WM_SETCURSOR, window.address(), (HTCLIENT as isize) | ((WM_MOUSEMOVE as isize) << 16));

    assert_eq!(result, 1);
    assert!(MessageLog::seen_of(WM_SETCURSOR).is_empty());
    assert_eq!(seen(), vec!["cursor"]);
}

#[test]
fn no_cursor_shape_leaves_wm_setcursor_to_the_game() {
    let (_harness, window) = start();

    window.send(WM_SETCURSOR, window.address(), (HTCLIENT as isize) | ((WM_MOUSEMOVE as isize) << 16));

    assert_eq!(MessageLog::seen_of(WM_SETCURSOR).len(), 1);
}

#[test]
fn the_requested_size_is_undone_at_disable() {
    let plain = EchoMod::build("plain", &[]);
    let (harness, window) = start();
    harness.install_mod("echo", "echo", &plain, "echo.dll", 2);
    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    let loader = ModLoader::new(harness.state);
    assert!(loader.load_all(&scan.mods).iter().all(|report| report.is_ok()));
    assert!(loader.enable_all().iter().all(|report| report.is_ok()));
    let size = AmmlSize { width: 1280, height: 720 };

    // SAFETY: the module view and the size live for the call.
    let status = unsafe { (window_table(&harness).request_size)(AmmlStr::from_str("@echo/echo"), &size) };
    assert_eq!(status, AmmlStatus::OK);
    window.pump();
    loader.disable_one(0);
    window.pump();

    assert_eq!(size_seen(), vec![SizeMessage::size_param(1280, 720), SizeMessage::size_param(WIDTH, HEIGHT)]);
}

#[test]
fn a_mod_that_holds_no_size_posts_nothing_at_disable() {
    let (harness, window) = start();
    harness.state.window().request_size(1, Some(AmmlSize { width: 1280, height: 720 })).unwrap();
    window.pump();

    harness.state.window().end_owner(0);
    window.pump();

    assert_eq!(size_seen(), vec![SizeMessage::size_param(1280, 720)]);
}

#[test]
fn a_size_with_a_zero_side_is_an_invalid_argument() {
    let (harness, _window) = start();

    let refused = harness.state.window().request_size(0, Some(AmmlSize { width: 0, height: 720 }));

    assert_eq!(refused, Err(AmmlStatus::INVALID_ARGUMENT));
}

#[test]
fn reads_before_attach_are_not_ready() {
    let harness = Harness::start();
    harness.state.window().offer();
    let table = window_table(&harness);
    let mut size = AmmlSize { width: 0, height: 0 };

    // SAFETY: `size` is a writable local.
    assert_eq!(unsafe { (table.client_size)(&mut size) }, AmmlStatus::NOT_READY);
    // SAFETY: the call takes no pointer.
    assert_eq!(unsafe { (table.replay_focus_loss)() }, AmmlStatus::NOT_READY);
}

#[test]
fn reads_of_a_window_area_not_offered_are_not_available() {
    let harness = Harness::start();
    let test = TestWindow::new(WIDTH, HEIGHT);
    assert!(harness.state.window().attach(test.address()));

    assert_eq!(harness.state.window().client_size(), Err(AmmlStatus::NOT_AVAILABLE));
    assert!(!harness.state.events().is_available("window.pointer", 1));
}

#[test]
fn reads_give_the_window_after_attach() {
    let (harness, _window) = start();
    let service = harness.state.window();

    assert_eq!(service.client_size(), Ok(AmmlSize { width: WIDTH, height: HEIGHT }));
    assert_eq!(service.buffer_size(), Err(AmmlStatus::NOT_READY));
    service.set_buffer(AmmlSize { width: 2000, height: 1000 });
    assert_eq!(service.buffer_size(), Ok(AmmlSize { width: 2000, height: 1000 }));
    assert_eq!(service.to_buffer(amml_abi::AmmlPoint { x: 5.0, y: 5.0 }), Ok(amml_abi::AmmlPoint { x: 10.0, y: 10.0 }));
    assert!(service.dpi().unwrap() > 0);
    assert_eq!(service.is_foreground(), Ok(false));
}

#[test]
fn a_second_window_is_ignored_while_the_first_exists() {
    let (harness, _window) = start();
    let other = TestWindow::new(WIDTH, HEIGHT);

    assert!(!harness.state.window().attach(other.address()));
    assert_ne!(harness.state.window().hwnd(), Some(other.address()));
}

#[test]
fn a_destroyed_window_is_replaced_by_the_next_attach() {
    let (harness, window) = start();
    drop(window);
    let other = TestWindow::new(640, 480);

    assert!(harness.state.window().attach(other.address()));
    assert_eq!(harness.state.window().hwnd(), Some(other.address()));
    assert_eq!(harness.state.window().client_size(), Ok(AmmlSize { width: 640, height: 480 }));
}

// One line per `loader.shutdown` the test listener saw.
static SHUTDOWNS: Mutex<Vec<String>> = Mutex::new(Vec::new());

fn shutdowns() -> Vec<String> {
    SHUTDOWNS.lock().unwrap().clone()
}

// Records `shutdown alive=<game_alive> client=<context client> game_thread=<bool> close_reached=<bool>`;
// `close_reached` tells whether the game's procedure had the `WM_CLOSE` already.
unsafe extern "C" fn on_shutdown(
    _: *mut AmmlEventHeader,
    context: *const AmmlEventContext,
    _: *mut c_void,
) -> AmmlStatus {
    // SAFETY: the loader passes a live context for the call.
    let context = unsafe { &*context };
    let game_thread = LoaderState::get().unwrap().events().gate().on_game_thread();
    let close_reached = !MessageLog::seen_of(WM_CLOSE).is_empty();
    SHUTDOWNS.lock().unwrap().push(format!(
        "shutdown alive={} client={:#x} game_thread={game_thread} close_reached={close_reached}",
        context.game_alive, context.client as usize
    ));
    AmmlStatus::OK
}

// A loader with the session adapter and a listener on `loader.shutdown`, after one frame on this thread and one
// update of client A (the primary client).
fn start_session() -> Harness {
    let harness = Harness::start_with_entries(&SessionGame::entries(&[]));
    SessionAdapter::install(harness.state);
    harness.state.window().offer();
    let listener = ModListener { owner: 40, callback: on_shutdown, user: 0 };
    harness.state.events().listen("loader.shutdown", 1, AmmlBand::NORMAL, listener).unwrap();
    SessionGame::frame();
    SessionGame::update(&CLIENT_A);
    harness
}

// A test window under the loader's subclass on this thread, with the recorder as the game's procedure.
fn attach_window(state: &LoaderState) -> TestWindow {
    let test = TestWindow::new(WIDTH, HEIGHT);
    MessageLog::install(test.hwnd);
    assert!(state.window().attach(test.address()));
    MessageLog::clear();
    test
}

// Loads and enables the events-mod fixture as `@events/events`.
fn enable_events_mod(harness: &Harness) -> ModLoader {
    let plain = EventsMod::build("plain", &[]);
    harness.install_mod("events", "events", &plain, "events.dll", 3);
    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    let loader = ModLoader::new(harness.state);
    assert!(loader.load_all(&scan.mods).iter().all(|report| report.is_ok()));
    assert!(loader.enable_all().iter().all(|report| report.is_ok()));
    loader
}

fn client_a() -> String {
    format!("{:#x}", SessionGame::client(&CLIENT_A) as usize)
}

#[test]
fn close_on_the_game_thread_runs_shutdown_with_the_game_alive() {
    let harness = start_session();
    let loader = enable_events_mod(&harness);
    let window = attach_window(harness.state);

    window.send(WM_CLOSE, 0, 0);

    let client = client_a();
    assert_eq!(shutdowns(), [format!("shutdown alive=1 client={client} game_thread=true close_reached=false")]);
    assert_eq!(MessageLog::seen_of(WM_CLOSE).len(), 1);
    assert_eq!(loader.phases(), [("@events/events".to_string(), ModPhase::Disabled)]);
    let disables = EventsMod::lines_of(&harness.mods(), "events", "disable listen");
    assert_eq!(disables, [format!("events disable listen status={} game_alive=1", AmmlStatus::FAILED.0)]);
    assert!(!harness.state.events().gate().is_open());
}

#[test]
fn close_on_another_thread_runs_shutdown_at_the_next_frame() {
    let harness = start_session();
    let window_thread = std::thread::spawn(|| {
        let window = attach_window(LoaderState::get().unwrap());
        window.send(WM_CLOSE, 0, 0);
        MessageLog::seen_of(WM_CLOSE).len()
    });
    let deadline = Instant::now() + Duration::from_secs(10);
    while !harness.state.shutdown().is_queued() {
        assert!(Instant::now() < deadline, "the close was never queued");
        std::thread::yield_now();
    }
    assert!(shutdowns().is_empty());

    SessionGame::frame();
    let closes = window_thread.join().unwrap();

    let client = client_a();
    assert_eq!(shutdowns(), [format!("shutdown alive=1 client={client} game_thread=true close_reached=false")]);
    assert_eq!(closes, 1);
    assert!(!harness.state.events().gate().is_open());
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "queued shutdown finished");
    assert!(log.contains("shutdown queued: the window thread is not the game thread"), "{log}");
}

#[test]
fn a_queued_close_the_game_thread_never_runs_reaches_the_game_after_the_wait() {
    let harness = start_session();
    let window_thread = std::thread::spawn(|| {
        let window = attach_window(LoaderState::get().unwrap());
        let started = Instant::now();
        window.send(WM_CLOSE, 0, 0);
        (started.elapsed(), MessageLog::seen_of(WM_CLOSE).len())
    });

    let (waited, closes) = window_thread.join().unwrap();

    assert!(waited >= WindowProc::SHUTDOWN_WAIT, "{waited:?}");
    assert_eq!(closes, 1);
    assert!(shutdowns().is_empty());
    assert!(harness.state.shutdown().is_queued());
    assert!(harness.state.shutdown().is_closing());
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "queued shutdown did not finish in time");
    assert!(log.contains("limit_ms"), "{log}");

    SessionGame::frame();

    let client = client_a();
    assert_eq!(shutdowns(), [format!("shutdown alive=1 client={client} game_thread=true close_reached=true")]);
}

#[test]
fn shutdown_runs_once() {
    let harness = start_session();
    let window = attach_window(harness.state);

    window.send(WM_CLOSE, 0, 0);
    window.send(WM_CLOSE, 0, 0);
    ModLoader::new(harness.state).shutdown();

    let client = client_a();
    assert_eq!(shutdowns(), [format!("shutdown alive=1 client={client} game_thread=true close_reached=false")]);
    assert_eq!(MessageLog::seen_of(WM_CLOSE).len(), 2);
}

#[test]
fn an_end_session_runs_shutdown_only_when_the_session_ends() {
    let harness = start_session();
    let window = attach_window(harness.state);

    window.send(WM_ENDSESSION, 0, 0);
    assert!(shutdowns().is_empty());
    window.send(WM_ENDSESSION, 1, 0);

    assert_eq!(shutdowns().len(), 1);
    assert_eq!(MessageLog::seen_of(WM_ENDSESSION).len(), 2);
    assert!(!harness.state.events().gate().is_open());
}

#[test]
fn no_size_is_posted_during_shutdown() {
    let harness = start_session();
    let _loader = enable_events_mod(&harness);
    let window = attach_window(harness.state);
    let size = AmmlSize { width: 1280, height: 720 };
    // SAFETY: the module view and the size live for the call.
    let status = unsafe { (window_table(&harness).request_size)(AmmlStr::from_str("@events/events"), &size) };
    assert_eq!(status, AmmlStatus::OK);
    window.pump();

    window.send(WM_CLOSE, 0, 0);
    window.pump();

    assert_eq!(size_seen(), vec![SizeMessage::size_param(1280, 720)]);
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "render size not restored");
    assert!(log.contains("the loader shuts down"), "{log}");
}

#[test]
fn a_size_request_while_closing_is_not_ready() {
    let harness = start_session();
    let window = attach_window(harness.state);

    window.send(WM_CLOSE, 0, 0);
    let refused = harness.state.window().request_size(0, Some(AmmlSize { width: 1280, height: 720 }));
    window.pump();

    assert_eq!(refused, Err(AmmlStatus::NOT_READY));
    assert!(size_seen().is_empty());
}

// One line per `world.left` or `loader.shutdown` the destroy listeners saw, in order.
static DESTROY_SEEN: Mutex<Vec<String>> = Mutex::new(Vec::new());

fn destroy_seen() -> Vec<String> {
    DESTROY_SEEN.lock().unwrap().clone()
}

// Records `<event> alive=<game_alive> destroy_reached=<bool>`; `user` 0 is `world.left`, 1 is `loader.shutdown`.
// `destroy_reached` tells whether the game's procedure had the `WM_DESTROY` already.
unsafe extern "C" fn on_closing(
    _: *mut AmmlEventHeader,
    context: *const AmmlEventContext,
    user: *mut c_void,
) -> AmmlStatus {
    // SAFETY: the loader passes a live context for the call.
    let context = unsafe { &*context };
    let event = if user.is_null() { "world.left" } else { "loader.shutdown" };
    let destroy_reached = !MessageLog::seen_of(WM_DESTROY).is_empty();
    DESTROY_SEEN
        .lock()
        .unwrap()
        .push(format!("{event} alive={} destroy_reached={destroy_reached}", context.game_alive));
    AmmlStatus::OK
}

// `start_session` with a joined world (epoch 1) and the destroy listeners on `world.left` and `loader.shutdown`.
fn start_joined_session() -> Harness {
    let harness = start_session();
    for (user, event) in ["world.left", "loader.shutdown"].into_iter().enumerate() {
        let listener = ModListener { owner: 41, callback: on_closing, user };
        harness.state.events().listen(event, 1, AmmlBand::NORMAL, listener).unwrap();
    }
    SessionGame::set_player(0x5000);
    SessionGame::start_join(&CLIENT_A, [0; 7]);
    SessionGame::update(&CLIENT_A);
    assert_eq!(harness.state.events().gate().epoch().0, 1);
    harness
}

#[test]
fn a_window_destroyed_without_a_close_leaves_the_world_and_shuts_down() {
    let harness = start_joined_session();
    let loader = enable_events_mod(&harness);
    let window = attach_window(harness.state);

    drop(window);

    assert_eq!(
        destroy_seen(),
        ["world.left alive=1 destroy_reached=false", "loader.shutdown alive=1 destroy_reached=false"]
    );
    let client = client_a();
    assert_eq!(shutdowns(), [format!("shutdown alive=1 client={client} game_thread=true close_reached=false")]);
    assert_eq!(MessageLog::seen_of(WM_DESTROY).len(), 1);
    assert_eq!(loader.phases(), [("@events/events".to_string(), ModPhase::Disabled)]);
    assert!(!harness.state.events().gate().is_open());
    assert_eq!(harness.state.events().gate().epoch().0, 0);
    assert_eq!(harness.state.window().hwnd(), None);
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "game window destroyed, the loader forgets it");
    assert!(log.contains("game window destroyed without a close, the loader shuts down"), "{log}");
}

#[test]
fn a_destroy_after_a_close_does_not_shut_down_again() {
    let harness = start_joined_session();
    let window = attach_window(harness.state);

    window.send(WM_CLOSE, 0, 0);
    drop(window);

    assert_eq!(
        destroy_seen(),
        ["world.left alive=1 destroy_reached=false", "loader.shutdown alive=1 destroy_reached=false"]
    );
    assert_eq!(shutdowns().len(), 1);
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "game window destroyed, the loader forgets it");
    assert!(!log.contains("game window destroyed without a close"), "{log}");
}

#[test]
fn a_close_or_destroy_after_a_destroy_does_not_shut_down_again() {
    let harness = start_joined_session();
    let window = attach_window(harness.state);

    window.send(WM_DESTROY, 0, 0);
    window.send(WM_CLOSE, 0, 0);
    window.send(WM_DESTROY, 0, 0);
    drop(window);
    ModLoader::new(harness.state).shutdown();

    assert_eq!(
        destroy_seen(),
        ["world.left alive=1 destroy_reached=false", "loader.shutdown alive=1 destroy_reached=false"]
    );
    assert_eq!(shutdowns().len(), 1);
    assert_eq!(MessageLog::seen_of(WM_DESTROY).len(), 3);
    assert_eq!(MessageLog::seen_of(WM_CLOSE).len(), 1);
}

#[test]
fn a_last_resort_nc_destroy_shuts_down_before_the_window_is_forgotten() {
    let harness = start_joined_session();
    let window = attach_window(harness.state);

    window.send(WM_NCDESTROY, 0, 0);

    assert_eq!(
        destroy_seen(),
        ["world.left alive=1 destroy_reached=false", "loader.shutdown alive=1 destroy_reached=false"]
    );
    assert_eq!(harness.state.window().hwnd(), None);
    assert_eq!(MessageLog::seen_of(WM_NCDESTROY).len(), 1);
}

#[test]
fn a_new_window_does_not_keep_the_pairing_of_the_old_one() {
    let (harness, old) = start();
    CANCEL_DOWNS.store(true, Ordering::SeqCst);
    old.send(WM_LBUTTONDOWN, 1, TestWindow::point(10, 20));
    drop(old);
    CANCEL_DOWNS.store(false, Ordering::SeqCst);
    let new = attach_window(harness.state);

    new.send(WM_LBUTTONUP, 0, TestWindow::point(10, 20));

    assert_eq!(MessageLog::seen_of(WM_LBUTTONUP).len(), 1);
}

// The procedure the later subclasser replaced, the calls of the later procedure and its deepest nesting.
static LATER_PREVIOUS: AtomicIsize = AtomicIsize::new(0);
static LATER_CALLS: AtomicUsize = AtomicUsize::new(0);
static LATER_DEPTH: AtomicUsize = AtomicUsize::new(0);
static LATER_RECURSED: AtomicBool = AtomicBool::new(false);

// A module that subclasses the game window after the loader, as an overlay does for its input: it passes every
// message on to the procedure it replaced. A nested call means the chain calls itself; it is refused and noted.
unsafe extern "system" fn later_proc(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let depth = LATER_DEPTH.fetch_add(1, Ordering::SeqCst);
    if depth > 0 {
        LATER_RECURSED.store(true, Ordering::SeqCst);
        LATER_DEPTH.fetch_sub(1, Ordering::SeqCst);
        return LRESULT(0);
    }
    LATER_CALLS.fetch_add(1, Ordering::SeqCst);
    let previous = LATER_PREVIOUS.load(Ordering::SeqCst);
    // SAFETY: `previous` is the procedure `SetWindowLongPtrW` gave back for this window.
    let previous = unsafe { std::mem::transmute::<isize, WNDPROC>(previous) };
    // SAFETY: the arguments of a live message, passed to the window's previous procedure.
    let result = unsafe { CallWindowProcW(previous, hwnd, message, wparam, lparam) };
    LATER_DEPTH.fetch_sub(1, Ordering::SeqCst);
    result
}

fn subclass_later(window: &TestWindow) {
    // SAFETY: `later_proc` has the signature of a window procedure and lives for the process.
    let previous = unsafe { SetWindowLongPtrW(window.hwnd, GWLP_WNDPROC, later_proc as *const () as isize) };
    LATER_PREVIOUS.store(previous, Ordering::SeqCst);
}

fn procedure_of(window: &TestWindow) -> usize {
    // SAFETY: the window is live; the call only reads its procedure.
    unsafe { GetWindowLongPtrW(window.hwnd, GWLP_WNDPROC) as usize }
}

#[test]
fn a_rebind_under_a_later_subclass_keeps_the_chain_and_does_not_recurse() {
    let (harness, window) = start();
    subclass_later(&window);
    assert_eq!(LATER_PREVIOUS.load(Ordering::SeqCst) as usize, WindowProc::address());

    assert!(harness.state.window().attach(window.address()));
    assert!(harness.state.window().attach(window.address()));
    window.send(WM_MOUSEMOVE, 0, TestWindow::point(10, 10));

    assert_eq!(procedure_of(&window), later_proc as *const () as usize);
    assert!(!LATER_RECURSED.load(Ordering::SeqCst));
    assert_eq!(LATER_CALLS.load(Ordering::SeqCst), 1);
    let pointers: Vec<String> = seen().into_iter().filter(|line| line.starts_with("pointer kind=0")).collect();
    assert_eq!(pointers.len(), 1, "{pointers:?}");
    assert_eq!(MessageLog::seen_of(WM_MOUSEMOVE).len(), 1);
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "another procedure subclassed the game window");
    assert_eq!(log.matches("game window subclassed").count(), 1, "{log}");
}

#[test]
fn a_destroyed_window_is_forgotten_at_nc_destroy() {
    let (harness, window) = start();
    subclass_later(&window);

    drop(window);

    assert_eq!(harness.state.window().hwnd(), None);
    assert_eq!(harness.state.window().client_size(), Err(AmmlStatus::NOT_READY));
    let other = TestWindow::new(640, 480);
    assert!(harness.state.window().attach(other.address()));
    assert_eq!(procedure_of(&other), WindowProc::address());
}

#[test]
fn a_clip_request_after_the_owner_was_ended_is_refused() {
    let (harness, _window) = start();
    let window = harness.state.window();

    assert_eq!(window.clip_cursor(0, Some(amml_runtime::ClipKind::Centre), || false), Err(AmmlStatus::REFUSED));
    assert_eq!(window.clip_cursor(0, Some(amml_runtime::ClipKind::Centre), || true), Ok(()));
    window.end_owner(0);
    assert_eq!(window.clip_cursor(0, None, || false), Ok(()), "dropping a request needs no live mod");
}

#[test]
fn a_disabled_mod_gets_no_clip_from_the_table() {
    let plain = EchoMod::build("plain", &[]);
    let (harness, _window) = start();
    harness.install_mod("echo", "echo", &plain, "echo.dll", 2);
    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    let loader = ModLoader::new(harness.state);
    assert!(loader.load_all(&scan.mods).iter().all(|report| report.is_ok()));
    assert!(loader.enable_all().iter().all(|report| report.is_ok()));
    let clip_cursor = window_table(&harness).clip_cursor.unwrap();
    let module = AmmlStr::from_str("@echo/echo");

    // SAFETY: the module view lives for the call.
    assert_eq!(unsafe { clip_cursor(module, AmmlCursorClip::CENTRE) }, AmmlStatus::OK);
    loader.disable_one(0);
    // SAFETY: as above.
    assert_eq!(unsafe { clip_cursor(module, AmmlCursorClip::CENTRE) }, AmmlStatus::REFUSED);
}

#[test]
fn an_unknown_clip_value_is_an_invalid_argument() {
    let (harness, _window) = start();
    let clip_cursor = window_table(&harness).clip_cursor.unwrap();

    // SAFETY: the module view lives for the call.
    assert_eq!(
        unsafe { clip_cursor(AmmlStr::from_str("@echo/echo"), AmmlCursorClip(9)) },
        AmmlStatus::INVALID_ARGUMENT
    );
}
