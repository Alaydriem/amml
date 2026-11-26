use std::ffi::c_void;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

use amml::{ModListener, OverlayService};
use amml_abi::{
    AmmlApi, AmmlBand, AmmlDrawSurface, AmmlEventContext, AmmlEventHeader, AmmlOverlayTable, AmmlServices, AmmlStatus,
    AmmlThread, EventCallbackFn, FramePresent, RenderDeviceReset, SurfaceLost, SwapChainResized,
};
use windows::Win32::Foundation::HMODULE;
use windows::Win32::Graphics::Direct3D::{D3D_DRIVER_TYPE_WARP, D3D_FEATURE_LEVEL_11_0};
use windows::Win32::Graphics::Direct3D11::{
    D3D11_BLEND_DESC, D3D11_CPU_ACCESS_READ, D3D11_CREATE_DEVICE_FLAG, D3D11_MAP_READ, D3D11_MAPPED_SUBRESOURCE,
    D3D11_SDK_VERSION, D3D11_TEXTURE2D_DESC, D3D11_USAGE, D3D11_USAGE_DEFAULT, D3D11_USAGE_STAGING, D3D11_VIEWPORT,
    D3D11CreateDevice, ID3D11BlendState, ID3D11Device, ID3D11DeviceContext, ID3D11RenderTargetView, ID3D11Texture2D,
};
use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_FORMAT_UNKNOWN, DXGI_SAMPLE_DESC};
use windows::Win32::Graphics::Dxgi::{
    DXGI_ERROR_DEVICE_REMOVED, DXGI_SWAP_CHAIN_FLAG, IDXGIAdapter, IDXGISwapChain, IDXGISwapChain3,
};
use windows::Win32::UI::WindowsAndMessaging::{SWP_NOMOVE, SWP_NOZORDER, SetWindowPos};
use windows::core::{IUnknown, Interface};

use crate::support::{D3d12Window, Harness, LogFile, SwapWindow};

// One line per event a listener saw.
static SEEN: Mutex<Vec<String>> = Mutex::new(Vec::new());
// The overlay table of the loader, for the listeners.
static TABLE: AtomicUsize = AtomicUsize::new(0);
// A texture the copy listener fills (a raw `ID3D11Texture2D` with one reference of the test), 0 before.
static FILLED: AtomicUsize = AtomicUsize::new(0);
// A texture of another device (a raw `ID3D11Texture2D` with one reference of the test).
static FOREIGN: AtomicUsize = AtomicUsize::new(0);
// The swap chain a listener resizes (a raw `IDXGISwapChain` the test keeps alive).
static CHAIN: AtomicUsize = AtomicUsize::new(0);
// How many times the resizing listener still resizes.
static RESIZES_LEFT: AtomicU32 = AtomicU32::new(0);

const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const GAME_VIEWPORT: D3D11_VIEWPORT =
    D3D11_VIEWPORT { TopLeftX: 10.0, TopLeftY: 20.0, Width: 30.0, Height: 40.0, MinDepth: 0.0, MaxDepth: 1.0 };
const LISTENER_VIEWPORT: D3D11_VIEWPORT =
    D3D11_VIEWPORT { TopLeftX: 0.0, TopLeftY: 0.0, Width: 64.0, Height: 64.0, MinDepth: 0.0, MaxDepth: 1.0 };

fn seen() -> Vec<String> {
    SEEN.lock().unwrap().clone()
}

fn note(line: String) {
    SEEN.lock().unwrap().push(line);
}

fn lines(prefix: &str) -> Vec<String> {
    seen().into_iter().filter(|line| line.starts_with(prefix)).collect()
}

// The surface of a `FramePresent`.
fn surface<'a>(event: *mut AmmlEventHeader) -> &'a AmmlDrawSurface {
    // SAFETY: the loader raises `overlay.frame_present` with a live `FramePresent`.
    unsafe { &(*event.cast::<FramePresent>()).surface }
}

fn device_of(surface: &AmmlDrawSurface) -> ID3D11Device {
    // SAFETY: a surface with a device lends a live `ID3D11Device` for the event.
    unsafe { ID3D11Device::from_raw_borrowed(&surface.device) }.unwrap().clone()
}

fn context_of(surface: &AmmlDrawSurface) -> ID3D11DeviceContext {
    // SAFETY: a surface with a context lends a live `ID3D11DeviceContext` for the event.
    unsafe { ID3D11DeviceContext::from_raw_borrowed(&surface.context) }.unwrap().clone()
}

fn clear(surface: &AmmlDrawSurface, colour: &[f32; 4]) {
    // SAFETY: a surface with a target lends a live render target view for the event.
    let target = unsafe { ID3D11RenderTargetView::from_raw_borrowed(&surface.target) }.unwrap();
    // SAFETY: the context and the view are live; the colour lives for the call.
    unsafe { context_of(surface).ClearRenderTargetView(target, colour) };
}

fn describe(surface: &AmmlDrawSurface) -> String {
    format!(
        "{}x{} format={} wrapped={} epoch={} target_null={} device_null={}",
        surface.width,
        surface.height,
        surface.format,
        surface.wrapped,
        surface.device_epoch,
        surface.target.is_null(),
        surface.device.is_null()
    )
}

fn table() -> &'static AmmlOverlayTable {
    let table = TABLE.load(Ordering::SeqCst);
    // SAFETY: the test stored the loader's overlay table, which lives for the process.
    unsafe { &*std::ptr::with_exposed_provenance::<AmmlOverlayTable>(table) }
}

fn copy(dest: *mut c_void) -> AmmlStatus {
    // SAFETY: `dest` is null or a live texture of the test.
    unsafe { (table().copy_back_buffer)(dest) }
}

fn texture(device: &ID3D11Device, width: u32, height: u32, usage: D3D11_USAGE) -> ID3D11Texture2D {
    let staging = usage == D3D11_USAGE_STAGING;
    let desc = D3D11_TEXTURE2D_DESC {
        Width: width,
        Height: height,
        MipLevels: 1,
        ArraySize: 1,
        Format: DXGI_FORMAT_R8G8B8A8_UNORM,
        SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
        Usage: usage,
        BindFlags: 0,
        CPUAccessFlags: if staging { D3D11_CPU_ACCESS_READ.0 as u32 } else { 0 },
        MiscFlags: 0,
    };
    let mut texture = None;
    // SAFETY: `desc` and the out pointer live for the call.
    unsafe { device.CreateTexture2D(&desc, None, Some(&mut texture)) }.unwrap();
    texture.unwrap()
}

// The first pixel of `source` (R8G8B8A8), read through a staging copy on `device`.
fn first_pixel(device: &ID3D11Device, source: &ID3D11Texture2D) -> [u8; 4] {
    let mut desc = D3D11_TEXTURE2D_DESC::default();
    // SAFETY: the texture is live and `desc` is writable.
    unsafe { source.GetDesc(&mut desc) };
    let staging = texture(device, desc.Width, desc.Height, D3D11_USAGE_STAGING);
    // SAFETY: the device is live.
    let context = unsafe { device.GetImmediateContext() }.unwrap();
    let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
    // SAFETY: both textures are live and alike; the mapped memory is read before the unmap.
    unsafe {
        context.CopyResource(&staging, source);
        context.Map(&staging, 0, D3D11_MAP_READ, 0, Some(&mut mapped)).unwrap();
        let bytes = std::slice::from_raw_parts(mapped.pData.cast::<u8>(), 4);
        let pixel = [bytes[0], bytes[1], bytes[2], bytes[3]];
        context.Unmap(&staging, 0);
        pixel
    }
}

fn game_device(chain: &IDXGISwapChain) -> (ID3D11Device, ID3D11DeviceContext) {
    // SAFETY: the chain is live and was made on a D3D11 device.
    let device: ID3D11Device = unsafe { chain.GetDevice() }.unwrap();
    // SAFETY: the device is live.
    let context = unsafe { device.GetImmediateContext() }.unwrap();
    (device, context)
}

fn blend_state(device: &ID3D11Device) -> ID3D11BlendState {
    let mut state = None;
    // SAFETY: the description and the out pointer live for the call.
    unsafe { device.CreateBlendState(&D3D11_BLEND_DESC::default(), Some(&mut state)) }.unwrap();
    state.unwrap()
}

// The viewport and blend state bound on `context` now.
fn bound_state(context: &ID3D11DeviceContext) -> (D3D11_VIEWPORT, Option<ID3D11BlendState>) {
    let mut count = 1u32;
    let mut viewport = D3D11_VIEWPORT::default();
    let mut blend = None;
    // SAFETY: the out pointers are writable locals sized for one viewport.
    unsafe {
        context.RSGetViewports(&mut count, Some(&mut viewport));
        context.OMGetBlendState(Some(&mut blend), None, None);
    }
    (viewport, blend)
}

unsafe extern "C" fn on_present(
    event: *mut AmmlEventHeader,
    context: *const AmmlEventContext,
    _: *mut c_void,
) -> AmmlStatus {
    // SAFETY: the loader passes a live context.
    let thread = unsafe { (*context).thread };
    note(format!("present {} thread={}", describe(surface(event)), thread.0));
    AmmlStatus::OK
}

unsafe extern "C" fn on_present_clearing(
    event: *mut AmmlEventHeader,
    _: *const AmmlEventContext,
    _: *mut c_void,
) -> AmmlStatus {
    let surface = surface(event);
    note(format!("present {}", describe(surface)));
    if !surface.target.is_null() {
        clear(surface, &RED);
    }
    AmmlStatus::OK
}

unsafe extern "C" fn on_present_setting_state(
    event: *mut AmmlEventHeader,
    _: *const AmmlEventContext,
    _: *mut c_void,
) -> AmmlStatus {
    let surface = surface(event);
    let context = context_of(surface);
    let blend = blend_state(&device_of(surface));
    // SAFETY: the context and the state are live.
    unsafe {
        context.RSSetViewports(Some(&[LISTENER_VIEWPORT]));
        context.OMSetBlendState(&blend, None, u32::MAX);
    }
    note("present set state".to_string());
    AmmlStatus::OK
}

unsafe extern "C" fn on_present_panicking(
    event: *mut AmmlEventHeader,
    _: *const AmmlEventContext,
    _: *mut c_void,
) -> AmmlStatus {
    let surface = surface(event);
    if !surface.context.is_null() {
        // SAFETY: the context is live.
        unsafe { context_of(surface).RSSetViewports(Some(&[LISTENER_VIEWPORT])) };
    }
    note("present panicked".to_string());
    AmmlStatus::PANICKED
}

unsafe extern "C" fn on_present_copying(
    event: *mut AmmlEventHeader,
    _: *const AmmlEventContext,
    _: *mut c_void,
) -> AmmlStatus {
    let surface = surface(event);
    clear(surface, &RED);
    let device = device_of(surface);
    let small = texture(&device, 1, 1, D3D11_USAGE_DEFAULT);
    let staging = texture(&device, surface.width, surface.height, D3D11_USAGE_STAGING);
    let filled = texture(&device, surface.width, surface.height, D3D11_USAGE_DEFAULT);
    let foreign = std::ptr::with_exposed_provenance_mut::<c_void>(FOREIGN.load(Ordering::SeqCst));
    note(format!(
        "copy null={} small={} staging={} foreign={} back_buffer={} same={}",
        copy(std::ptr::null_mut()).0,
        copy(small.as_raw()).0,
        copy(staging.as_raw()).0,
        copy(foreign).0,
        copy(surface.back_buffer).0,
        copy(filled.as_raw()).0
    ));
    FILLED.store(filled.into_raw().expose_provenance(), Ordering::SeqCst);
    AmmlStatus::OK
}

unsafe extern "C" fn on_present_scribbling(
    event: *mut AmmlEventHeader,
    _: *const AmmlEventContext,
    _: *mut c_void,
) -> AmmlStatus {
    // SAFETY: the loader raises `overlay.frame_present` with a live, writable `FramePresent`.
    let present = unsafe { &mut *event.cast::<FramePresent>() };
    present.surface.target = std::ptr::null_mut();
    present.surface.width = 1;
    present.surface.device_epoch = 99;
    AmmlStatus::OK
}

unsafe extern "C" fn on_present_resizing(
    event: *mut AmmlEventHeader,
    _: *const AmmlEventContext,
    _: *mut c_void,
) -> AmmlStatus {
    note(format!("present {}", describe(surface(event))));
    if RESIZES_LEFT.load(Ordering::SeqCst) == 0 {
        return AmmlStatus::OK;
    }
    RESIZES_LEFT.fetch_sub(1, Ordering::SeqCst);
    let raw = std::ptr::with_exposed_provenance_mut::<c_void>(CHAIN.load(Ordering::SeqCst));
    // SAFETY: the test keeps the chain alive while it presents.
    let chain = unsafe { IDXGISwapChain::from_raw_borrowed(&raw) }.unwrap();
    // SAFETY: the chain is live; 0 buffers and an unknown format keep what it has.
    let result = unsafe { chain.ResizeBuffers(0, 128, 96, DXGI_FORMAT_UNKNOWN, DXGI_SWAP_CHAIN_FLAG(0)) };
    note(format!("inner resize ok={}", result.is_ok()));
    AmmlStatus::OK
}

unsafe extern "C" fn on_lost(
    event: *mut AmmlEventHeader,
    context: *const AmmlEventContext,
    _: *mut c_void,
) -> AmmlStatus {
    // SAFETY: the loader raises `overlay.surface_lost` with a `SurfaceLost` and a live context.
    let (lost, context) = unsafe { (&*event.cast::<SurfaceLost>(), &*context) };
    note(format!(
        "lost reason={} epoch={} phase={} thread={}",
        lost.reason.0, lost.device_epoch, lost.header.phase.0, context.thread.0
    ));
    AmmlStatus::OK
}

unsafe extern "C" fn on_reset(event: *mut AmmlEventHeader, _: *const AmmlEventContext, _: *mut c_void) -> AmmlStatus {
    // SAFETY: the loader raises `overlay.render_device_reset` with a `RenderDeviceReset`.
    let reset = unsafe { &*event.cast::<RenderDeviceReset>() };
    note(format!("reset epoch={}", reset.device_epoch));
    AmmlStatus::OK
}

unsafe extern "C" fn on_resized(event: *mut AmmlEventHeader, _: *const AmmlEventContext, _: *mut c_void) -> AmmlStatus {
    // SAFETY: the loader raises `overlay.swap_chain_resized` with a `SwapChainResized`.
    let resized = unsafe { &*event.cast::<SwapChainResized>() };
    note(format!("resized ok={} {}x{}", resized.ok, resized.width, resized.height));
    AmmlStatus::OK
}

// A loader with the overlay installed, the gate open, `present` on `FramePresent` (when given) and a recorder on
// `SurfaceLost`, `RenderDeviceReset` and `SwapChainResized`.
fn start(present: Option<EventCallbackFn>) -> Harness {
    let harness = Harness::start();
    assert!(OverlayService::install(harness.state));
    if let Some(present) = present {
        listen(&harness, "overlay.frame_present", 0, AmmlBand::NORMAL, present);
    }
    listen(&harness, "overlay.surface_lost", 0, AmmlBand::NORMAL, on_lost);
    listen(&harness, "overlay.render_device_reset", 0, AmmlBand::NORMAL, on_reset);
    listen(&harness, "overlay.swap_chain_resized", 0, AmmlBand::NORMAL, on_resized);
    // SAFETY: the loader's tables live for the process.
    let services = unsafe { AmmlApi::services(harness.state.api()) }.unwrap();
    // SAFETY: as above.
    let overlay = unsafe { AmmlServices::overlay(services) }.unwrap();
    TABLE.store(overlay.expose_provenance(), Ordering::SeqCst);
    harness.state.events().gate().open();
    harness
}

fn listen(harness: &Harness, event: &str, owner: usize, band: AmmlBand, callback: EventCallbackFn) {
    let listener = ModListener { owner, callback, user: 0 };
    harness.state.events().listen(event, 1, band, listener).unwrap();
}

fn log(harness: &Harness, needle: &str) -> String {
    LogFile::wait_for(&harness.logs().join("loader.log"), needle)
}

#[test]
fn a_d3d11_chain_gives_a_surface_with_its_size_and_format() {
    let _harness = start(Some(on_present));
    let window = SwapWindow::new();

    window.present();
    window.present();

    let size = SwapWindow::SIZE;
    let format = DXGI_FORMAT_R8G8B8A8_UNORM.0;
    let thread = AmmlThread::RENDER.0;
    let line = format!(
        "present {size}x{size} format={format} wrapped=0 epoch=1 target_null=false device_null=false thread={thread}"
    );
    assert_eq!(lines("present"), vec![line.clone(), line]);
    assert!(lines("reset").is_empty(), "{:?}", seen());
}

#[test]
fn a_d3d12_chain_gives_a_wrapped_surface_after_the_queue_is_seen() {
    let harness = start(Some(on_present));
    let window = D3d12Window::new();

    // The first Present has no queue yet. DXGI's own Present submits on the chain's queue, which the hook sees, so
    // the second frame already has a surface; the game's submit changes nothing after that.
    window.present();
    window.present();
    window.submit();
    window.present();

    let size = SwapWindow::SIZE;
    let format = DXGI_FORMAT_R8G8B8A8_UNORM.0;
    let thread = AmmlThread::RENDER.0;
    let empty = format!("present 0x0 format=0 wrapped=0 epoch=0 target_null=true device_null=true thread={thread}");
    let wrapped = format!(
        "present {size}x{size} format={format} wrapped=1 epoch=1 target_null=false device_null=false thread={thread}"
    );
    assert_eq!(lines("present"), vec![empty, wrapped.clone(), wrapped]);
    let text = log(&harness, "overlay queue");
    assert_eq!(text.matches("waiting for the command queue").count(), 1, "{text}");
    assert!(text.contains("D3D12 through D3D11On12"), "{text}");
    let queue_line = text.lines().find(|line| line.contains("overlay queue")).unwrap_or_else(|| panic!("{text}"));
    assert!(queue_line.contains("\"same_device\":true"), "{queue_line}");
    let bound_line = text.lines().find(|line| line.contains("overlay bound")).unwrap();
    assert!(bound_line.contains("\"d3d12\":true"), "{bound_line}");

    // The wrappers are released before ResizeBuffers, so it succeeds, and the next frame has the new size.
    window.resize(128, 96).unwrap();
    window.present();
    assert_eq!(lines("resized"), vec!["resized ok=1 128x96".to_string()]);
    assert!(lines("present").last().unwrap().starts_with("present 128x96 "), "{:?}", seen());
}

#[test]
fn the_pipeline_state_is_restored_after_listeners() {
    let _harness = start(Some(on_present_setting_state));
    let window = SwapWindow::new();
    let (device, context) = game_device(&window.chain);
    let game_blend = blend_state(&device);
    // SAFETY: the context and the state are live.
    unsafe {
        context.RSSetViewports(Some(&[GAME_VIEWPORT]));
        context.OMSetBlendState(&game_blend, None, u32::MAX);
    }

    window.present();

    assert_eq!(lines("present"), vec!["present set state".to_string()]);
    let (viewport, blend) = bound_state(&context);
    assert_eq!(viewport, GAME_VIEWPORT);
    assert_eq!(blend.map(|blend| blend.as_raw()), Some(game_blend.as_raw()));
}

#[test]
fn a_new_device_raises_surface_lost_then_reset() {
    let _harness = start(Some(on_present));
    let window = SwapWindow::new();
    window.present();

    let window = window.replace_chain();
    window.present();

    let thread = AmmlThread::RENDER.0;
    let order: Vec<String> =
        seen().into_iter().map(|line| line.split(" thread=").next().unwrap().to_string()).collect();
    let size = SwapWindow::SIZE;
    let format = DXGI_FORMAT_R8G8B8A8_UNORM.0;
    let present = |epoch: u32| {
        format!("present {size}x{size} format={format} wrapped=0 epoch={epoch} target_null=false device_null=false")
    };
    assert_eq!(
        order,
        vec![present(1), "lost reason=0 epoch=1 phase=0".to_string(), "reset epoch=2".to_string(), present(2)]
    );
    assert!(lines("lost").iter().all(|line| line.ends_with(&format!("thread={thread}"))), "{:?}", seen());
}

#[test]
fn copy_back_buffer_refuses_another_size_or_outside_present() {
    let _harness = start(Some(on_present_copying));
    let window = SwapWindow::new();
    let (device, _) = game_device(&window.chain);
    let foreign_device = warp_device();
    let foreign = texture(&foreign_device, SwapWindow::SIZE, SwapWindow::SIZE, D3D11_USAGE_DEFAULT);
    FOREIGN.store(foreign.as_raw().expose_provenance(), Ordering::SeqCst);

    window.present();

    let invalid = AmmlStatus::INVALID_ARGUMENT.0;
    let refused = AmmlStatus::REFUSED.0;
    let ok = AmmlStatus::OK.0;
    assert_eq!(
        lines("copy"),
        vec![format!(
            "copy null={invalid} small={refused} staging={refused} foreign={refused} back_buffer={refused} same={ok}"
        )]
    );
    let raw = std::ptr::with_exposed_provenance_mut::<c_void>(FILLED.swap(0, Ordering::SeqCst));
    // SAFETY: the listener stored the texture with one reference, which this takes over.
    let filled = unsafe { ID3D11Texture2D::from_raw(raw) };
    assert_eq!(first_pixel(&device, &filled), [255, 0, 0, 255]);
    // Outside a present, on the test thread.
    assert_eq!(copy(filled.as_raw()), AmmlStatus::REFUSED);
    assert_eq!(copy(std::ptr::null_mut()), AmmlStatus::INVALID_ARGUMENT);
}

#[test]
fn resize_with_a_listener_drawing_does_not_fail() {
    let _harness = start(Some(on_present_clearing));
    let window = SwapWindow::on_window(200, 100);
    window.present();

    window.resize(128, 96).unwrap();
    window.present();
    window.resize(0, 0).unwrap();
    window.present();
    // A minimised window: its client area is 0 x 0.
    // SAFETY: the window is live.
    unsafe { SetWindowPos(window.window.hwnd, None, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOZORDER) }.unwrap();
    window.resize(0, 0).unwrap();
    window.present();

    let resized = lines("resized");
    assert_eq!(resized.len(), 3, "{resized:?}");
    assert!(resized.iter().all(|line| line.starts_with("resized ok=1 ")), "{resized:?}");
    assert_eq!(resized[0], "resized ok=1 128x96");
    assert_eq!(resized[1], "resized ok=1 200x100");
    let presents = lines("present");
    assert!(presents[1].starts_with("present 128x96 "), "{presents:?}");
    assert!(presents[2].starts_with("present 200x100 "), "{presents:?}");
    let (width, height) = window.buffer_size();
    let last = presents.last().unwrap();
    if width == 0 || height == 0 {
        assert!(last.contains("target_null=true"), "{presents:?}");
    } else {
        assert!(last.starts_with(&format!("present {width}x{height} ")), "{presents:?} {width}x{height}");
    }
}

#[test]
fn a_panicking_listener_still_restores_the_state_and_presents_once() {
    let harness = start(Some(on_present_panicking));
    let window = SwapWindow::new();
    let (_, context) = game_device(&window.chain);
    // SAFETY: the context is live.
    unsafe { context.RSSetViewports(Some(&[GAME_VIEWPORT])) };
    let reached = window.dxgi_present_count();

    window.present();

    assert_eq!(lines("present"), vec!["present panicked".to_string()]);
    assert_eq!(bound_state(&context).0, GAME_VIEWPORT);
    assert_eq!(window.dxgi_present_count(), reached + 1);
    assert!(harness.state.events().deferred().is_queued(0));
}

#[test]
fn a_panicking_listener_on_d3d12_leaves_no_back_buffer_held() {
    let _harness = start(Some(on_present_panicking));
    let window = D3d12Window::new();
    window.submit();
    window.present();

    window.resize(128, 96).unwrap();

    assert_eq!(lines("present"), vec!["present panicked".to_string()]);
    assert_eq!(lines("resized"), vec!["resized ok=1 128x96".to_string()]);
}

#[test]
fn a_listener_cannot_change_the_surface_for_the_next_listener() {
    let harness = start(None);
    listen(&harness, "overlay.frame_present", 0, AmmlBand::WRITE, on_present_scribbling);
    listen(&harness, "overlay.frame_present", 1, AmmlBand::NORMAL, on_present);
    let window = SwapWindow::new();

    window.present();

    let size = SwapWindow::SIZE;
    let present = lines("present");
    assert_eq!(present.len(), 1);
    assert!(present[0].starts_with(&format!("present {size}x{size} ")), "{present:?}");
    assert!(present[0].contains("epoch=1 target_null=false"), "{present:?}");
}

#[test]
fn a_resize_inside_frame_present_is_refused_and_frames_go_on() {
    let harness = start(Some(on_present_resizing));
    let window = SwapWindow::new();
    CHAIN.store(window.chain.as_raw().expose_provenance(), Ordering::SeqCst);
    RESIZES_LEFT.store(1, Ordering::SeqCst);

    window.present();
    window.present();
    window.resize(128, 96).unwrap();
    window.present();

    let size = SwapWindow::SIZE;
    assert_eq!(lines("inner"), vec!["inner resize ok=false".to_string()]);
    assert_eq!(lines("resized"), vec![format!("resized ok=0 {size}x{size}"), "resized ok=1 128x96".to_string()]);
    let presents = lines("present");
    assert_eq!(presents.len(), 3, "{presents:?}");
    assert!(presents[1].starts_with(&format!("present {size}x{size} ")), "{presents:?}");
    assert!(presents[2].starts_with("present 128x96 "), "{presents:?}");
    let text = log(&harness, "inside FramePresent");
    assert!(text.contains("swap chain resize inside FramePresent"), "{text}");
}

#[test]
fn a_removed_device_raises_surface_lost_once() {
    let _harness = start(Some(on_present));
    let window = D3d12Window::new();
    window.submit();
    window.present();

    window.wait();
    window.remove_device();
    let removed = window.present_result();
    window.present();

    assert_eq!(removed, DXGI_ERROR_DEVICE_REMOVED);
    let lost = lines("lost");
    assert_eq!(lost.len(), 1, "{:?}", seen());
    assert!(lost[0].starts_with("lost reason=0 epoch=1 phase=0"), "{lost:?}");
}

#[test]
fn a_new_device_after_a_removed_one_binds_again() {
    let harness = start(Some(on_present));
    let window = D3d12Window::new();
    window.submit();
    window.present();
    window.wait();
    window.remove_device();
    window.present();

    let window = window.rebuild();
    window.submit();
    window.present();
    window.present();

    let text = log(&harness, "device_epoch\":2");
    assert_eq!(text.matches("overlay bound").count(), 2, "{text}");
    assert_eq!(lines("reset"), vec!["reset epoch=2".to_string()]);
    let last = lines("present").pop().unwrap();
    assert!(last.contains("wrapped=1 epoch=2 target_null=false"), "{last}");
}

#[test]
fn a_d3d12_game_can_rebuild_its_chain_while_a_listener_draws() {
    let _harness = start(Some(on_present_clearing));
    let window = D3d12Window::new();
    window.submit();
    window.present();
    window.present();
    window.wait();

    // Fails with E_ACCESSDENIED while the loader holds a buffer of the old chain.
    let window = window.rebuild();
    window.submit();
    window.present();
    window.present();

    // The loader's device kept the D3D12 device alive, so the game's new chain is on the same device (D3D12 gives one
    // device per adapter): a chain replacement.
    assert_eq!(lines("lost"), vec!["lost reason=1 epoch=1 phase=0 thread=1".to_string()], "{:?}", seen());
    assert_eq!(lines("reset"), vec!["reset epoch=2".to_string()]);
    let last = lines("present").pop().unwrap();
    assert!(last.contains("wrapped=1 epoch=2 target_null=false"), "{last}");
}

#[test]
fn copy_back_buffer_is_not_available_without_the_overlay() {
    let harness = Harness::start();
    // SAFETY: the loader's tables live for the process.
    let services = unsafe { AmmlApi::services(harness.state.api()) }.unwrap();
    // SAFETY: as above.
    let overlay = unsafe { AmmlServices::overlay(services) }.unwrap();
    TABLE.store(overlay.expose_provenance(), Ordering::SeqCst);

    assert_eq!(copy(std::ptr::null_mut()), AmmlStatus::NOT_AVAILABLE);
    // SAFETY: the table is live.
    assert_eq!(unsafe { (table().present_count)() }, 0);
}

#[test]
fn resize_buffers1_raises_one_resize() {
    let _harness = start(Some(on_present));
    let window = SwapWindow::new();
    window.present();
    let chain: IDXGISwapChain3 = window.chain.cast().unwrap();
    let masks = [1u32, 1];
    let queues: [Option<IUnknown>; 2] = [None, None];

    // SAFETY: the chain is live; a D3D11 chain takes no queues, and the arrays hold one entry per buffer.
    let result = unsafe {
        chain.ResizeBuffers1(0, 128, 96, DXGI_FORMAT_UNKNOWN, DXGI_SWAP_CHAIN_FLAG(0), masks.as_ptr(), queues.as_ptr())
    };

    let resized = lines("resized");
    if result.is_ok() {
        assert_eq!(resized, vec!["resized ok=1 128x96".to_string()]);
    } else {
        assert_eq!(resized.len(), 1, "{resized:?}");
    }
}

#[test]
fn without_a_frame_present_listener_no_device_is_made() {
    let harness = start(None);
    let window = SwapWindow::new();

    window.present();
    window.present();

    let text = log(&harness, "overlay bound");
    assert!(text.contains("overlay bound"), "{text}");
    assert!(!text.contains("overlay device ready"), "{text}");
    assert_eq!(OverlayService::present_count(), 2);
}

// A WARP D3D11 device of its own, for a texture of another device.
fn warp_device() -> ID3D11Device {
    let mut device = None;
    // SAFETY: the out pointer lives for the call.
    unsafe {
        D3D11CreateDevice(
            None::<&IDXGIAdapter>,
            D3D_DRIVER_TYPE_WARP,
            HMODULE::default(),
            D3D11_CREATE_DEVICE_FLAG(0),
            Some(&[D3D_FEATURE_LEVEL_11_0]),
            D3D11_SDK_VERSION,
            Some(&mut device),
            None,
            None,
        )
    }
    .unwrap();
    device.unwrap()
}
