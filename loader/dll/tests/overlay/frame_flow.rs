use std::ffi::c_void;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};

use amml::{ModListener, OverlayService, ProbeTargets};
use amml_abi::{
    AmmlBand, AmmlEventContext, AmmlEventHeader, AmmlPhase, AmmlSize, AmmlStatus, AmmlThread, FramePresent,
    FramePresented, SwapChainResize, SwapChainResized,
};
use windows::Win32::Graphics::Dxgi::{DXGI_PRESENT, DXGI_PRESENT_TEST};

use crate::support::{Harness, LogFile, SwapWindow};

// One line per event a listener saw.
static SEEN: Mutex<Vec<String>> = Mutex::new(Vec::new());
// The resize listener writes this size when it is not u32::MAX.
static WRITE_WIDTH: AtomicU32 = AtomicU32::new(u32::MAX);
static WRITE_HEIGHT: AtomicU32 = AtomicU32::new(u32::MAX);

fn seen() -> Vec<String> {
    SEEN.lock().unwrap().clone()
}

fn note(line: String) {
    SEEN.lock().unwrap().push(line);
}

fn presents() -> usize {
    seen().iter().filter(|line| line.starts_with("present ")).count()
}

unsafe extern "C" fn on_present(
    event: *mut AmmlEventHeader,
    context: *const AmmlEventContext,
    _: *mut c_void,
) -> AmmlStatus {
    // SAFETY: the loader raises `overlay.frame_present` with a `FramePresent` and a live context.
    let (present, context) = unsafe { (&*event.cast::<FramePresent>(), &*context) };
    note(format!(
        "present frame={} phase={} thread={} target_null={} surface_size={}",
        present.frame_id,
        present.header.phase.0,
        context.thread.0,
        present.surface.target.is_null(),
        present.surface.size
    ));
    AmmlStatus::OK
}

unsafe extern "C" fn on_presented(
    event: *mut AmmlEventHeader,
    _: *const AmmlEventContext,
    _: *mut c_void,
) -> AmmlStatus {
    // SAFETY: the loader raises `overlay.frame_presented` with a `FramePresented`.
    let presented = unsafe { &*event.cast::<FramePresented>() };
    note(format!(
        "presented frame={} phase={} result={:#x}",
        presented.frame_id, presented.header.phase.0, presented.result
    ));
    AmmlStatus::OK
}

unsafe extern "C" fn on_resize(event: *mut AmmlEventHeader, _: *const AmmlEventContext, _: *mut c_void) -> AmmlStatus {
    // SAFETY: the loader raises `overlay.swap_chain_resize` with a `SwapChainResize`.
    let resize = unsafe { &mut *event.cast::<SwapChainResize>() };
    note(format!("resize {}x{} phase={}", resize.width, resize.height, resize.header.phase.0));
    let (width, height) = (WRITE_WIDTH.load(Ordering::SeqCst), WRITE_HEIGHT.load(Ordering::SeqCst));
    if width != u32::MAX {
        resize.width = width;
        resize.height = height;
    }
    AmmlStatus::OK
}

unsafe extern "C" fn on_resized(event: *mut AmmlEventHeader, _: *const AmmlEventContext, _: *mut c_void) -> AmmlStatus {
    // SAFETY: the loader raises `overlay.swap_chain_resized` with a `SwapChainResized`.
    let resized = unsafe { &*event.cast::<SwapChainResized>() };
    note(format!(
        "resized ok={} {}x{} reserved={:?} phase={}",
        resized.ok, resized.width, resized.height, resized._reserved, resized.header.phase.0
    ));
    AmmlStatus::OK
}

// A loader with the overlay installed and a listener on each overlay event. The gate stays as `open` says.
fn start(open: bool) -> Harness {
    let harness = Harness::start();
    assert!(OverlayService::install(harness.state));
    let listen = |event: &str, callback| {
        let listener = ModListener { owner: 0, callback, user: 0 };
        harness.state.events().listen(event, 1, AmmlBand::NORMAL, listener).unwrap();
    };
    listen("overlay.frame_present", on_present);
    listen("overlay.frame_presented", on_presented);
    listen("overlay.swap_chain_resize", on_resize);
    listen("overlay.swap_chain_resized", on_resized);
    if open {
        harness.state.events().gate().open();
    }
    harness
}

fn write_resize(width: u32, height: u32) {
    WRITE_WIDTH.store(width, Ordering::SeqCst);
    WRITE_HEIGHT.store(height, Ordering::SeqCst);
}

#[test]
fn no_chain_is_bound_before_the_gate_opens() {
    let harness = start(false);
    let window = SwapWindow::new();

    window.present();
    window.present();
    assert_eq!(presents(), 0);
    assert_eq!(OverlayService::present_count(), 0);

    harness.state.events().gate().open();
    window.present();

    assert_eq!(presents(), 1);
    assert_eq!(OverlayService::present_count(), 1);
}

#[test]
fn a_second_window_chain_is_ignored() {
    let _harness = start(true);
    let game = SwapWindow::new();
    let other = SwapWindow::new();

    game.present();
    other.present();
    other.present();
    assert_eq!(presents(), 1);
    game.present();

    assert_eq!(presents(), 2);
    assert_eq!(OverlayService::present_count(), 2);
}

#[test]
fn present1_and_present_count_as_one() {
    let _harness = start(true);
    let window = SwapWindow::new();

    window.present1();
    assert_eq!(presents(), 1, "{:?}", seen());
    assert_eq!(OverlayService::present_count(), 1);
    window.present();

    assert_eq!(presents(), 2);
    assert_eq!(OverlayService::present_count(), 2);
}

#[test]
fn a_test_present_is_not_a_frame() {
    let _harness = start(true);
    let window = SwapWindow::new();

    window.present();
    let _ = window.present_with(DXGI_PRESENT_TEST);

    assert_eq!(presents(), 1);
    assert_eq!(OverlayService::present_count(), 1);
}

#[test]
fn frame_present_comes_before_and_frame_presented_after_with_the_result() {
    let _harness = start(true);
    let window = SwapWindow::new();

    let result = window.present_with(DXGI_PRESENT(0));

    let before = AmmlPhase::BEFORE.0;
    let after = AmmlPhase::AFTER.0;
    let thread = AmmlThread::RENDER.0;
    assert_eq!(
        seen(),
        vec![
            format!("present frame=1 phase={before} thread={thread} target_null=false surface_size=64"),
            format!("presented frame=1 phase={after} result={:#x}", result.0),
        ]
    );
}

#[test]
fn the_bind_attaches_the_window_and_sets_the_buffer_size() {
    let harness = start(true);
    let window = SwapWindow::on_window(200, 100);

    window.present();

    let service = harness.state.window();
    assert_eq!(service.hwnd(), Some(window.address()));
    let size = SwapWindow::SIZE;
    assert_eq!(service.buffer_size().unwrap(), AmmlSize { width: size, height: size });
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "overlay bound");
    assert!(log.contains("overlay hooks installed"), "{log}");
    assert!(log.contains("overlay bound"), "{log}");
}

#[test]
fn a_written_resize_size_reaches_resize_buffers() {
    let harness = start(true);
    let window = SwapWindow::new();
    window.present();
    write_resize(640, 360);

    window.resize(128, 96).unwrap();

    assert_eq!(window.buffer_size(), (640, 360));
    let lines = seen();
    let before = AmmlPhase::BEFORE.0;
    let after = AmmlPhase::AFTER.0;
    assert_eq!(lines[2], format!("resize 128x96 phase={before}"), "{lines:?}");
    assert_eq!(lines[3], format!("resized ok=1 640x360 reserved=[0, 0, 0] phase={after}"), "{lines:?}");
    assert_eq!(harness.state.window().buffer_size().unwrap(), AmmlSize { width: 640, height: 360 });
}

#[test]
fn resize_to_zero_takes_the_window_size() {
    let _harness = start(true);
    let window = SwapWindow::on_window(200, 100);
    window.present();
    write_resize(0, 0);

    window.resize(0, 0).unwrap();
    assert_eq!(window.buffer_size(), (200, 100));

    // A written 0 keeps what the game asked.
    window.resize(300, 150).unwrap();
    assert_eq!(window.buffer_size(), (300, 150));
    let resized: Vec<String> = seen().into_iter().filter(|line| line.starts_with("resized")).collect();
    let after = AmmlPhase::AFTER.0;
    assert_eq!(
        resized,
        vec![
            format!("resized ok=1 200x100 reserved=[0, 0, 0] phase={after}"),
            format!("resized ok=1 300x150 reserved=[0, 0, 0] phase={after}"),
        ]
    );
}

#[test]
fn a_failed_resize_reports_the_old_size_and_frames_go_on() {
    let harness = start(true);
    let window = SwapWindow::new();
    window.present();

    let held = window.hold_buffer();
    assert!(window.resize(128, 96).is_err());
    drop(held);

    let size = SwapWindow::SIZE;
    let after = AmmlPhase::AFTER.0;
    let resized: Vec<String> = seen().into_iter().filter(|line| line.starts_with("resized")).collect();
    assert_eq!(resized, vec![format!("resized ok=0 {size}x{size} reserved=[0, 0, 0] phase={after}")]);
    assert_eq!(harness.state.window().buffer_size().unwrap(), AmmlSize { width: size, height: size });

    window.present();
    assert_eq!(presents(), 2);
    window.resize(128, 96).unwrap();
    assert_eq!(harness.state.window().buffer_size().unwrap(), AmmlSize { width: 128, height: 96 });
}

#[test]
fn a_resize_of_another_chain_raises_nothing() {
    let _harness = start(true);
    let game = SwapWindow::new();
    let other = SwapWindow::new();
    game.present();
    write_resize(640, 360);

    other.resize(128, 96).unwrap();

    assert_eq!(other.buffer_size(), (128, 96));
    assert!(seen().iter().all(|line| !line.starts_with("resize")), "{:?}", seen());
}

#[test]
fn a_destroyed_window_shuts_down_and_no_later_chain_binds() {
    let harness = start(true);
    let first = SwapWindow::new();
    first.present();
    drop(first);

    let second = SwapWindow::on_window(200, 100);
    second.present();

    assert!(harness.state.shutdown().is_closing());
    assert_eq!(presents(), 1);
    assert_eq!(harness.state.window().hwnd(), None);
}

#[test]
fn while_the_loader_shuts_down_no_event_is_raised_and_the_original_runs() {
    let harness = start(true);
    let window = SwapWindow::new();
    window.present();
    let reached = window.dxgi_present_count();

    let run = harness.state.shutdown().run().unwrap();
    window.present();
    write_resize(640, 360);
    window.resize(128, 96).unwrap();
    drop(run);

    assert_eq!(window.dxgi_present_count(), reached + 1);
    assert_eq!(window.buffer_size(), (128, 96));
    assert_eq!(seen().len(), 2, "{:?}", seen());
    assert_eq!(OverlayService::present_count(), 1);
}

#[test]
fn a_failed_install_offers_no_overlay_or_window_need() {
    let harness = Harness::start();
    let targets =
        ProbeTargets { present: 0, resize_buffers: 0, present1: 0, resize_buffers1: 0, execute_command_lists: None };

    assert!(!OverlayService::install_targets(harness.state, &targets));

    let events = harness.state.events();
    assert!(!events.is_available("overlay.frame_present", 1));
    assert!(!events.is_available("overlay.swap_chain_resized", 1));
    assert!(!events.is_available("overlay.surface_lost", 1));
    assert!(!events.is_available("window.pointer", 1));
    assert!(!harness.state.areas().is_available(OverlayService::SURFACE_AREA, 1));
    assert!(!harness.state.areas().is_available("window.read", 1));
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "overlay hooks not installed");
    assert!(log.contains("overlay hooks not installed"), "{log}");
}

#[test]
fn a_successful_install_offers_the_overlay_and_window_needs() {
    let harness = start(false);

    let events = harness.state.events();
    for event in ["overlay.frame_present", "overlay.frame_presented", "overlay.swap_chain_resize", "window.size"] {
        assert!(events.is_available(event, 1), "{event}");
    }
    assert!(harness.state.areas().is_available(OverlayService::SURFACE_AREA, 1));
    assert!(harness.state.areas().is_available("window.read", 1));
    assert!(events.is_available("overlay.surface_lost", 1));
    assert!(events.is_available("overlay.render_device_reset", 1));
}
