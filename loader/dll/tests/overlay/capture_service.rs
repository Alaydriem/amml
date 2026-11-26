use std::ffi::c_void;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use amml::{CaptureService, ModListener, ModLoader, OverlayService, OverlayShared, SessionAdapter};
use amml_abi::{
    AmmlApi, AmmlBand, AmmlCaptureFrame, AmmlCaptureRequest, AmmlDrawSurface, AmmlEventContext, AmmlEventHeader,
    AmmlOverlayTable, AmmlServices, AmmlShotLayers, AmmlStatus, AmmlStr, EventCallbackFn, FramePresent,
    ScreenshotTaken, ScreenshotWritten,
};
use amml_runtime::ModRegistry;
use windows::Win32::Graphics::Direct3D11::{
    ID3D11Device, ID3D11DeviceContext, ID3D11RenderTargetView, ID3D11Texture2D,
};
use windows::core::Interface;

use crate::support::{D3d12Window, EchoMod, Harness, LogFile, SessionGame, SwapWindow};

// One line per screenshot event a listener saw: "<owner> taken|written <id> ok=<ok>".
static SEEN: Mutex<Vec<String>> = Mutex::new(Vec::new());
// The overlay table of the loader.
static TABLE: AtomicUsize = AtomicUsize::new(0);

const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const BLUE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
// A mod index no loaded mod has; the drawing listeners use it.
const DRAWER: usize = 9;

fn seen() -> Vec<String> {
    SEEN.lock().unwrap().clone()
}

fn table() -> &'static AmmlOverlayTable {
    let table = TABLE.load(Ordering::SeqCst);
    // SAFETY: the test stored the loader's overlay table, which lives for the process.
    unsafe { &*std::ptr::with_exposed_provenance::<AmmlOverlayTable>(table) }
}

fn clear_surface(event: *mut AmmlEventHeader, colour: &[f32; 4]) {
    // SAFETY: the loader raises `overlay.frame_present` with a live `FramePresent`.
    let surface: &AmmlDrawSurface = unsafe { &(*event.cast::<FramePresent>()).surface };
    if surface.target.is_null() {
        return;
    }
    // SAFETY: a surface with a target lends a live context and render target view for the event.
    let (context, target) = unsafe {
        (
            ID3D11DeviceContext::from_raw_borrowed(&surface.context).unwrap(),
            ID3D11RenderTargetView::from_raw_borrowed(&surface.target).unwrap(),
        )
    };
    // SAFETY: the context and the view are live; the colour lives for the call.
    unsafe { context.ClearRenderTargetView(target, colour) };
}

unsafe extern "C" fn on_present_red(
    event: *mut AmmlEventHeader,
    _: *const AmmlEventContext,
    _: *mut c_void,
) -> AmmlStatus {
    clear_surface(event, &RED);
    AmmlStatus::OK
}

unsafe extern "C" fn on_present_blue(
    event: *mut AmmlEventHeader,
    _: *const AmmlEventContext,
    _: *mut c_void,
) -> AmmlStatus {
    clear_surface(event, &BLUE);
    AmmlStatus::OK
}

unsafe extern "C" fn on_taken(
    event: *mut AmmlEventHeader,
    _: *const AmmlEventContext,
    user: *mut c_void,
) -> AmmlStatus {
    // SAFETY: the loader raises `overlay.screenshot_taken` with a `ScreenshotTaken`.
    let taken = unsafe { &*event.cast::<ScreenshotTaken>() };
    SEEN.lock().unwrap().push(format!("{} taken {} ok={}", user as usize, taken.id, taken.ok));
    AmmlStatus::OK
}

unsafe extern "C" fn on_written(
    event: *mut AmmlEventHeader,
    _: *const AmmlEventContext,
    user: *mut c_void,
) -> AmmlStatus {
    // SAFETY: the loader raises `overlay.screenshot_written` with a `ScreenshotWritten`.
    let written = unsafe { &*event.cast::<ScreenshotWritten>() };
    SEEN.lock().unwrap().push(format!("{} written {} ok={}", user as usize, written.id, written.ok));
    AmmlStatus::OK
}

// A loader with the overlay installed and the gate open, and the screenshot events recorded for mods 0 and 1.
fn start() -> Harness {
    let harness = Harness::start();
    assert!(OverlayService::install(harness.state));
    for owner in [0, 1] {
        listen(&harness, "overlay.screenshot_taken", owner, AmmlBand::NORMAL, on_taken);
        listen(&harness, "overlay.screenshot_written", owner, AmmlBand::NORMAL, on_written);
    }
    // SAFETY: the loader's tables live for the process.
    let services = unsafe { AmmlApi::services(harness.state.api()) }.unwrap();
    // SAFETY: as above.
    let overlay = unsafe { AmmlServices::overlay(services) }.unwrap();
    TABLE.store(overlay.expose_provenance(), Ordering::SeqCst);
    harness.state.events().gate().open();
    harness
}

fn listen(harness: &Harness, event: &str, owner: usize, band: AmmlBand, callback: EventCallbackFn) {
    let listener = ModListener { owner, callback, user: owner };
    harness.state.events().listen(event, 1, band, listener).unwrap();
}

fn capture(harness: &Harness) -> &CaptureService {
    harness.state.capture()
}

// The game's frame: back buffer 0 cleared to green on the game's device, then presented.
fn present_game(window: &SwapWindow) {
    // SAFETY: the chain is live and was made on a D3D11 device; buffer 0 is the current back buffer of a flip chain.
    unsafe {
        let device: ID3D11Device = window.chain.GetDevice().unwrap();
        let context = device.GetImmediateContext().unwrap();
        let buffer: ID3D11Texture2D = window.chain.GetBuffer(0).unwrap();
        let mut view = None;
        device.CreateRenderTargetView(&buffer, None, Some(&mut view)).unwrap();
        context.ClearRenderTargetView(&view.unwrap(), &GREEN);
    }
    window.present();
}

// Presents game frames (raising the queued screenshot events on this thread after each) until `done` holds; at most
// 10 s.
fn present_until(harness: &Harness, mut present: impl FnMut(), mut done: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while !done() {
        assert!(Instant::now() < deadline, "timed out: {:?}", seen());
        present();
        harness.state.owner_events().drain(harness.state.events());
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn written(id: u64) -> impl Fn() -> bool {
    move || seen().iter().any(|line| line.contains(&format!("written {id} ")))
}

// The first pixel of the PNG at `path`, as RGB.
fn png_first_pixel(path: &Path) -> [u8; 3] {
    let decoder = png::Decoder::new(std::io::BufReader::new(File::open(path).unwrap()));
    let mut reader = decoder.read_info().unwrap();
    let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut pixels).unwrap();
    assert_eq!(info.color_type, png::ColorType::Rgb);
    [pixels[0], pixels[1], pixels[2]]
}

fn request(layers: AmmlShotLayers, ring: u8, max_width: u32, max_height: u32) -> AmmlCaptureRequest {
    AmmlCaptureRequest {
        size: AmmlCaptureRequest::SIZE_V1,
        layers,
        ring,
        _reserved: 0,
        every_nth_present: 1,
        max_width,
        max_height,
    }
}

// Presents with `present` until a frame of `stream` can be taken.
fn take_frame(harness: &Harness, stream: u64, mut present: impl FnMut()) -> AmmlCaptureFrame {
    let mut frame = None;
    present_until(harness, &mut present, || {
        frame = capture(harness).take(stream).ok();
        frame.is_some()
    });
    frame.unwrap()
}

// The first four bytes of a taken frame.
fn first_bgra(frame: &AmmlCaptureFrame) -> [u8; 4] {
    // SAFETY: a taken frame lends `len` bytes of pixels until it is given back.
    let pixels = unsafe { std::slice::from_raw_parts(frame.pixels, frame.len) };
    [pixels[0], pixels[1], pixels[2], pixels[3]]
}

fn shot_path(harness: &Harness, name: &str) -> PathBuf {
    harness.root.path().join(name)
}

#[test]
fn game_only_has_no_overlay_and_with_overlays_has_it() {
    let harness = start();
    listen(&harness, "overlay.frame_present", DRAWER, AmmlBand::NORMAL, on_present_red);
    listen(&harness, "overlay.frame_present", DRAWER, AmmlBand::LAST, on_present_blue);
    let window = SwapWindow::new();
    let game_path = shot_path(&harness, "game.png");
    let with_path = shot_path(&harness, "with.png");

    assert_eq!(capture(&harness).screenshot(0, &game_path, AmmlShotLayers::GAME_ONLY), Err(AmmlStatus::NOT_READY));
    present_game(&window);
    let game = capture(&harness).screenshot(0, &game_path, AmmlShotLayers::GAME_ONLY).unwrap();
    let with = capture(&harness).screenshot(0, &with_path, AmmlShotLayers::WITH_OVERLAYS).unwrap();
    present_until(&harness, || present_game(&window), || written(game)() && written(with)());

    assert_eq!(png_first_pixel(&game_path), [0, 255, 0]);
    // After the NORMAL band (red) and before LAST (blue).
    assert_eq!(png_first_pixel(&with_path), [255, 0, 0]);
    let lines = seen();
    assert!(lines.iter().all(|line| line.starts_with("0 ")), "{lines:?}");
    for id in [game, with] {
        let taken = lines.iter().position(|line| *line == format!("0 taken {id} ok=1"));
        let written = lines.iter().position(|line| *line == format!("0 written {id} ok=1"));
        assert!(taken.is_some() && written.is_some() && taken < written, "{lines:?}");
    }
    assert_eq!(lines.len(), 4, "{lines:?}");
    assert!(!harness.root.path().join("game.png.tmp").exists());
}

#[test]
fn an_unwritable_path_gives_written_false() {
    let harness = start();
    let window = SwapWindow::new();
    present_game(&window);
    let path = harness.root.path().join("missing").join("shot.png");

    let id = capture(&harness).screenshot(1, &path, AmmlShotLayers::WITH_OVERLAYS).unwrap();
    present_until(&harness, || present_game(&window), written(id));

    assert_eq!(seen(), vec![format!("1 taken {id} ok=1"), format!("1 written {id} ok=0")]);
    assert!(!path.exists());
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "screenshot not written");
    let line = log.lines().find(|line| line.contains("screenshot not written")).unwrap_or_else(|| panic!("{log}"));
    assert!(line.contains("missing") && line.contains("\"error\""), "{line}");
}

#[test]
fn a_relative_path_or_unknown_layers_is_an_invalid_argument() {
    let harness = start();
    let window = SwapWindow::new();
    present_game(&window);
    let absolute = shot_path(&harness, "shot.png");

    let relative = capture(&harness).screenshot(0, Path::new("shot.png"), AmmlShotLayers::GAME_ONLY);
    let unknown = capture(&harness).screenshot(0, &absolute, AmmlShotLayers(2));

    assert_eq!(relative, Err(AmmlStatus::INVALID_ARGUMENT));
    assert_eq!(unknown, Err(AmmlStatus::INVALID_ARGUMENT));
    for ring in [0, CaptureService::MAX_RING + 1] {
        let opened = capture(&harness).open_capture(0, &request(AmmlShotLayers::GAME_ONLY, ring, 0, 0));
        assert_eq!(opened, Err(AmmlStatus::INVALID_ARGUMENT), "ring {ring}");
    }
}

#[test]
fn a_slow_consumer_makes_the_stream_skip_not_wait() {
    let harness = start();
    let window = SwapWindow::new();
    present_game(&window);
    let stream = capture(&harness).open_capture(0, &request(AmmlShotLayers::GAME_ONLY, 2, 0, 0)).unwrap();

    for _ in 0..10 {
        let started = Instant::now();
        present_game(&window);
        assert!(started.elapsed() < Duration::from_secs(2));
    }

    let skipped = capture(&harness).skipped(stream).unwrap();
    assert!(skipped > 0, "skipped {skipped}");
    assert_eq!(capture(&harness).free_slots(stream), 0);
}

#[test]
fn a_taken_frame_holds_bgra_pixels_of_the_back_buffer() {
    let harness = start();
    listen(&harness, "overlay.frame_present", DRAWER, AmmlBand::NORMAL, on_present_red);
    let window = SwapWindow::new();
    present_game(&window);
    let stream = capture(&harness).open_capture(0, &request(AmmlShotLayers::WITH_OVERLAYS, 2, 0, 0)).unwrap();

    let frame = take_frame(&harness, stream, || present_game(&window));

    let size = SwapWindow::SIZE;
    assert_eq!((frame.width, frame.height), (size, size));
    assert!(frame.stride >= size * 4);
    assert!(frame.len >= (frame.stride * size) as usize);
    assert_eq!(first_bgra(&frame), [0, 0, 255, 255]);
    assert!(frame.present_index > 0);
    assert_eq!(capture(&harness).give_back(stream, frame.present_index), Ok(()));
}

#[test]
fn a_game_only_stream_with_no_listener_holds_the_game_picture() {
    let harness = start();
    let window = SwapWindow::new();
    present_game(&window);
    let stream = capture(&harness).open_capture(0, &request(AmmlShotLayers::GAME_ONLY, 2, 0, 0)).unwrap();

    let frame = take_frame(&harness, stream, || present_game(&window));

    assert_eq!(first_bgra(&frame), [0, 255, 0, 255]);
}

#[test]
fn closing_a_stream_with_a_taken_frame_frees_once() {
    let harness = start();
    let window = SwapWindow::new();
    present_game(&window);
    let stream = capture(&harness).open_capture(0, &request(AmmlShotLayers::GAME_ONLY, 2, 0, 0)).unwrap();
    let frame = take_frame(&harness, stream, || present_game(&window));

    assert_eq!(capture(&harness).close(stream), Ok(()));

    assert_eq!(capture(&harness).take(stream).err(), Some(AmmlStatus::STALE));
    assert_eq!(capture(&harness).close(stream), Err(AmmlStatus::STALE));
    assert_eq!(capture(&harness).give_back(stream, frame.present_index), Ok(()));
    assert_eq!(capture(&harness).give_back(stream, frame.present_index), Err(AmmlStatus::STALE));
    // The closed stream copies nothing more.
    present_game(&window);
    assert!(!capture(&harness).has_work());
}

#[test]
fn a_frame_over_the_limit_is_scaled_by_a_whole_factor() {
    let harness = start();
    let window = SwapWindow::new();
    present_game(&window);
    window.resize(640, 360).unwrap();
    present_game(&window);
    let stream = capture(&harness).open_capture(0, &request(AmmlShotLayers::GAME_ONLY, 2, 320, 200)).unwrap();

    let frame = take_frame(&harness, stream, || present_game(&window));

    assert_eq!((frame.width, frame.height), (320, 180));
    assert_eq!(frame.stride, 320 * 4);
    assert_eq!(first_bgra(&frame), [0, 255, 0, 255]);
}

#[test]
fn disable_closes_the_mods_streams() {
    let plain = EchoMod::build("plain", &[]);
    let harness = start();
    harness.install_mod("echo", "echo", &plain, "echo.dll", 2);
    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(&harness.mods()).unwrap();
    let loader = ModLoader::new(harness.state);
    assert!(loader.load_all(&scan.mods).iter().all(|report| report.is_ok()));
    assert!(loader.enable_all().iter().all(|report| report.is_ok()));
    let window = SwapWindow::new();
    present_game(&window);
    let wanted = request(AmmlShotLayers::GAME_ONLY, 2, 0, 0);
    let mut stream = 0;
    // SAFETY: the module view, the request and the out pointer live for the call.
    let opened = unsafe { (table().open_capture)(AmmlStr::from_str("@echo/echo"), &wanted, &mut stream) };
    assert_eq!(opened, AmmlStatus::OK);
    // SAFETY: a stream id needs no pointer.
    assert_eq!(unsafe { (table().capture_free_slots)(stream) }, 2);

    loader.disable_one(0);

    let mut frame = std::mem::MaybeUninit::<AmmlCaptureFrame>::uninit();
    // SAFETY: the out pointer is writable for the call.
    assert_eq!(unsafe { (table().capture_take)(stream, frame.as_mut_ptr()) }, AmmlStatus::STALE);
    // SAFETY: a stream id needs no pointer.
    assert_eq!(unsafe { (table().capture_free_slots)(stream) }, 0);
    assert!(!capture(&harness).has_work());
}

#[test]
fn a_d3d12_chain_gives_captured_frames() {
    let harness = start();
    listen(&harness, "overlay.frame_present", DRAWER, AmmlBand::NORMAL, on_present_red);
    let window = D3d12Window::new();
    window.present();
    window.present();
    let stream = capture(&harness).open_capture(0, &request(AmmlShotLayers::WITH_OVERLAYS, 2, 0, 0)).unwrap();

    let frame = take_frame(&harness, stream, || window.present());

    assert_eq!((frame.width, frame.height), (SwapWindow::SIZE, SwapWindow::SIZE));
    assert_eq!(first_bgra(&frame), [0, 0, 255, 255]);
    // The wrappers were released after the copy: a resize still works.
    assert_eq!(capture(&harness).give_back(stream, frame.present_index), Ok(()));
    window.resize(128, 96).unwrap();
}

// Presents a D3D12 frame with the kept command queue cleared first, as a game whose queue the loader never sees: the
// frame has no surface.
fn present_without_queue(window: &D3d12Window) {
    OverlayShared::global().queue().clear();
    window.present();
}

fn drain(harness: &Harness) {
    harness.state.owner_events().drain(harness.state.events());
}

#[test]
fn a_shot_on_a_d3d12_chain_with_no_queue_fails_after_its_presents() {
    let harness = start();
    listen(&harness, "overlay.frame_present", DRAWER, AmmlBand::NORMAL, on_present_red);
    let window = D3d12Window::new();
    present_without_queue(&window);
    let id = capture(&harness).screenshot(0, &shot_path(&harness, "never.png"), AmmlShotLayers::WITH_OVERLAYS).unwrap();

    for _ in 1..CaptureService::SHOT_PRESENTS {
        present_without_queue(&window);
    }
    drain(&harness);
    assert!(seen().is_empty(), "{:?}", seen());
    present_without_queue(&window);
    drain(&harness);

    assert_eq!(seen(), vec![format!("0 taken {id} ok=0"), format!("0 written {id} ok=0")]);
    assert!(!capture(&harness).has_work());
    let log = LogFile::wait_for(&harness.logs().join("loader.log"), "found no frame with a surface in time");
    assert!(log.contains("waiting for the command queue"), "{log}");
}

#[test]
fn a_shot_with_no_present_fails_after_the_wait() {
    let harness = start();
    let window = SwapWindow::new();
    present_game(&window);
    let id = capture(&harness).screenshot(1, &shot_path(&harness, "late.png"), AmmlShotLayers::GAME_ONLY).unwrap();

    capture(&harness).expire_shots(Instant::now());
    drain(&harness);
    assert!(seen().is_empty(), "{:?}", seen());
    capture(&harness).expire_shots(Instant::now() + CaptureService::SHOT_WAIT);
    drain(&harness);

    assert_eq!(seen(), vec![format!("1 taken {id} ok=0"), format!("1 written {id} ok=0")]);
}

#[test]
fn a_shot_or_a_stream_asked_while_closing_is_not_ready() {
    let harness = start();
    let window = SwapWindow::new();
    present_game(&window);

    ModLoader::new(harness.state).shutdown();

    let shot = capture(&harness).screenshot(0, &shot_path(&harness, "closing.png"), AmmlShotLayers::GAME_ONLY);
    let stream = capture(&harness).open_capture(0, &request(AmmlShotLayers::GAME_ONLY, 2, 0, 0));
    assert_eq!(shot, Err(AmmlStatus::NOT_READY));
    assert_eq!(stream, Err(AmmlStatus::NOT_READY));
}

// The status a screenshot asked from the `Shutdown` listener got.
static SHOT_IN_SHUTDOWN: Mutex<Option<AmmlStatus>> = Mutex::new(None);

unsafe extern "C" fn on_shutdown(_: *mut AmmlEventHeader, _: *const AmmlEventContext, _: *mut c_void) -> AmmlStatus {
    SEEN.lock().unwrap().push("shutdown".to_string());
    let state = amml::LoaderState::get().unwrap();
    let path = state.dirs().logs.join("in-shutdown.png");
    let status = state.capture().screenshot(0, &path, AmmlShotLayers::GAME_ONLY).err().unwrap_or(AmmlStatus::OK);
    *SHOT_IN_SHUTDOWN.lock().unwrap() = Some(status);
    AmmlStatus::OK
}

#[test]
fn a_waiting_shot_gets_both_events_before_shutdown() {
    let harness = start();
    harness.state.events().gate().note_game_thread();
    listen(&harness, "loader.shutdown", 0, AmmlBand::NORMAL, on_shutdown);
    let window = SwapWindow::new();
    present_game(&window);
    let id = capture(&harness).screenshot(0, &shot_path(&harness, "exit.png"), AmmlShotLayers::GAME_ONLY).unwrap();

    ModLoader::new(harness.state).shutdown_live(0);

    assert_eq!(seen(), vec![format!("0 taken {id} ok=0"), format!("0 written {id} ok=0"), "shutdown".to_string()]);
    assert_eq!(*SHOT_IN_SHUTDOWN.lock().unwrap(), Some(AmmlStatus::NOT_READY));
    assert!(!capture(&harness).has_work());
}

#[test]
fn a_shot_while_no_frame_presents_fails_at_a_game_frame_after_the_wait() {
    let harness = Harness::start_with_entries(&SessionGame::entries(&[]));
    SessionAdapter::install(harness.state);
    assert!(OverlayService::install(harness.state));
    listen(&harness, "overlay.screenshot_taken", 0, AmmlBand::NORMAL, on_taken);
    listen(&harness, "overlay.screenshot_written", 0, AmmlBand::NORMAL, on_written);
    SessionGame::frame();
    let window = SwapWindow::new();
    present_game(&window);
    let id = capture(&harness).screenshot(0, &shot_path(&harness, "minimised.png"), AmmlShotLayers::GAME_ONLY).unwrap();

    SessionGame::frame();
    assert!(seen().is_empty(), "{:?}", seen());
    std::thread::sleep(CaptureService::SHOT_WAIT);
    SessionGame::frame();

    assert_eq!(seen(), vec![format!("0 taken {id} ok=0"), format!("0 written {id} ok=0")]);
}
