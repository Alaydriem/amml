use std::ffi::c_void;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use amml::{ModListener, OverlayService, SurfaceCounts};
use amml_abi::{
    AmmlApi, AmmlBand, AmmlDrawSurface, AmmlEventContext, AmmlEventHeader, AmmlOverlayTable, AmmlServices, AmmlStatus,
    EventCallbackFn, FramePresent, SwapChainResized,
};
use windows::Win32::Graphics::Direct3D11::{
    D3D11_CPU_ACCESS_READ, D3D11_MAP_READ, D3D11_MAPPED_SUBRESOURCE, D3D11_TEXTURE2D_DESC, D3D11_USAGE,
    D3D11_USAGE_DEFAULT, D3D11_USAGE_STAGING, D3D11_VIEWPORT, ID3D11Device, ID3D11DeviceContext,
    ID3D11RenderTargetView, ID3D11Texture2D,
};
use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_SAMPLE_DESC};
use windows::Win32::Graphics::Dxgi::IDXGISwapChain;
use windows::core::Interface;

use crate::support::{D3d12Window, Harness, SwapWindow};

// One line per event a listener saw.
static SEEN: Mutex<Vec<String>> = Mutex::new(Vec::new());
// The overlay table of the loader, for the listeners.
static TABLE: AtomicUsize = AtomicUsize::new(0);
// A texture the copying listener fills (a raw `ID3D11Texture2D` with one reference of the test), 0 before.
static FILLED: AtomicUsize = AtomicUsize::new(0);

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

fn table() -> &'static AmmlOverlayTable {
    let table = TABLE.load(Ordering::SeqCst);
    // SAFETY: the test stored the loader's overlay table, which lives for the process.
    unsafe { &*std::ptr::with_exposed_provenance::<AmmlOverlayTable>(table) }
}

fn prepare() -> (AmmlStatus, AmmlDrawSurface) {
    let prepare = table().prepare_surface.unwrap();
    let mut out = std::mem::MaybeUninit::<AmmlDrawSurface>::zeroed();
    // SAFETY: `out` is writable for the call.
    let status = unsafe { prepare(out.as_mut_ptr()) };
    // SAFETY: all zero bytes are a valid surface (null pointers), and the loader writes a whole one on `OK`.
    (status, unsafe { out.assume_init() })
}

// The surface of a `FramePresent`.
fn surface<'a>(event: *mut AmmlEventHeader) -> &'a AmmlDrawSurface {
    // SAFETY: the loader raises `overlay.frame_present` with a live `FramePresent`.
    unsafe { &(*event.cast::<FramePresent>()).surface }
}

fn describe(surface: &AmmlDrawSurface) -> String {
    format!(
        "{}x{} wrapped={} epoch={} device_null={} context_null={} target_null={} back_buffer_null={}",
        surface.width,
        surface.height,
        surface.wrapped,
        surface.device_epoch,
        surface.device.is_null(),
        surface.context.is_null(),
        surface.target.is_null(),
        surface.back_buffer.is_null()
    )
}

fn context_of(surface: &AmmlDrawSurface) -> ID3D11DeviceContext {
    // SAFETY: a full surface lends a live `ID3D11DeviceContext` for the event.
    unsafe { ID3D11DeviceContext::from_raw_borrowed(&surface.context) }.unwrap().clone()
}

fn device_of(surface: &AmmlDrawSurface) -> ID3D11Device {
    // SAFETY: a surface with a device lends a live `ID3D11Device` for the event.
    unsafe { ID3D11Device::from_raw_borrowed(&surface.device) }.unwrap().clone()
}

fn clear(surface: &AmmlDrawSurface, colour: &[f32; 4]) {
    // SAFETY: a full surface lends a live render target view for the event.
    let target = unsafe { ID3D11RenderTargetView::from_raw_borrowed(&surface.target) }.unwrap();
    // SAFETY: the context and the view are live; the colour lives for the call.
    unsafe { context_of(surface).ClearRenderTargetView(target, colour) };
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
    let staging = texture(device, SwapWindow::SIZE, SwapWindow::SIZE, D3D11_USAGE_STAGING);
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

fn viewport(context: &ID3D11DeviceContext) -> D3D11_VIEWPORT {
    let mut count = 1u32;
    let mut viewport = D3D11_VIEWPORT::default();
    // SAFETY: the out pointers are writable locals sized for one viewport.
    unsafe { context.RSGetViewports(&mut count, Some(&mut viewport)) };
    viewport
}

fn game_context(chain: &IDXGISwapChain) -> (ID3D11Device, ID3D11DeviceContext) {
    // SAFETY: the chain is live and was made on a D3D11 device.
    let device: ID3D11Device = unsafe { chain.GetDevice() }.unwrap();
    // SAFETY: the device is live.
    let context = unsafe { device.GetImmediateContext() }.unwrap();
    (device, context)
}

unsafe extern "C" fn on_present_idle(
    event: *mut AmmlEventHeader,
    _: *const AmmlEventContext,
    _: *mut c_void,
) -> AmmlStatus {
    note(format!("idle {}", describe(surface(event))));
    AmmlStatus::OK
}

unsafe extern "C" fn on_present_asking(
    event: *mut AmmlEventHeader,
    _: *const AmmlEventContext,
    _: *mut c_void,
) -> AmmlStatus {
    let before = describe(surface(event));
    let (status, made) = prepare();
    let (again, twice) = prepare();
    note(format!(
        "asking before=[{before}] status={} made=[{}] again={} same_target={}",
        status.0,
        describe(&made),
        again.0,
        made.target == twice.target
    ));
    if status == AmmlStatus::OK {
        // SAFETY: the context of a made surface is live for the event.
        unsafe { context_of(&made).RSSetViewports(Some(&[LISTENER_VIEWPORT])) };
        clear(&made, &RED);
    }
    AmmlStatus::OK
}

unsafe extern "C" fn on_present_copying(
    event: *mut AmmlEventHeader,
    _: *const AmmlEventContext,
    _: *mut c_void,
) -> AmmlStatus {
    let surface = surface(event);
    let filled = texture(&device_of(surface), surface.width, surface.height, D3D11_USAGE_DEFAULT);
    // SAFETY: the texture is live for the call.
    let status = unsafe { (table().copy_back_buffer)(filled.as_raw()) };
    note(format!("copy status={} {}", status.0, describe(surface)));
    FILLED.store(filled.into_raw().expose_provenance(), Ordering::SeqCst);
    AmmlStatus::OK
}

unsafe extern "C" fn on_resized(event: *mut AmmlEventHeader, _: *const AmmlEventContext, _: *mut c_void) -> AmmlStatus {
    // SAFETY: the loader raises `overlay.swap_chain_resized` with a `SwapChainResized`.
    let resized = unsafe { &*event.cast::<SwapChainResized>() };
    note(format!("resized ok={} {}x{}", resized.ok, resized.width, resized.height));
    AmmlStatus::OK
}

// A loader with the overlay installed, the gate open, the overlay table kept for the listeners and a recorder on
// `SwapChainResized`.
fn start() -> Harness {
    let harness = Harness::start();
    assert!(OverlayService::install(harness.state));
    // SAFETY: the loader's tables live for the process.
    let services = unsafe { AmmlApi::services(harness.state.api()) }.unwrap();
    // SAFETY: as above.
    let overlay = unsafe { AmmlServices::overlay(services) }.unwrap();
    TABLE.store(overlay.expose_provenance(), Ordering::SeqCst);
    listen(&harness, "overlay.swap_chain_resized", 1, AmmlBand::NORMAL, on_resized);
    harness.state.events().gate().open();
    harness
}

fn listen(harness: &Harness, event: &str, version: u32, band: AmmlBand, callback: EventCallbackFn) {
    let listener = ModListener { owner: 0, callback, user: 0 };
    harness.state.events().listen(event, version, band, listener).unwrap();
}

fn present_listener(harness: &Harness, version: u32, band: AmmlBand, callback: EventCallbackFn) {
    listen(harness, "overlay.frame_present", version, band, callback);
}

fn counts_since(start: SurfaceCounts) -> SurfaceCounts {
    let now = OverlayService::surface_counts();
    SurfaceCounts {
        made: now.made - start.made,
        asked: now.asked - start.asked,
        state_saves: now.state_saves - start.state_saves,
    }
}

fn on_demand_surface(wrapped: u8) -> String {
    let size = SwapWindow::SIZE;
    format!(
        "{size}x{size} wrapped={wrapped} epoch=1 device_null=false context_null=true target_null=true \
         back_buffer_null=true"
    )
}

fn full_surface(wrapped: u8) -> String {
    let size = SwapWindow::SIZE;
    format!(
        "{size}x{size} wrapped={wrapped} epoch=1 device_null=false context_null=false target_null=false \
         back_buffer_null=false"
    )
}

#[test]
fn an_idle_on_demand_listener_on_d3d11_makes_no_surface() {
    let harness = start();
    present_listener(&harness, FramePresent::ON_DEMAND_VERSION, AmmlBand::NORMAL, on_present_idle);
    let window = SwapWindow::new();
    window.present();
    let counted = OverlayService::surface_counts();

    window.present();
    window.present();

    assert_eq!(counts_since(counted), SurfaceCounts::default());
    let idle = format!("idle {}", on_demand_surface(0));
    assert_eq!(lines("idle"), vec![idle.clone(), idle.clone(), idle]);
}

#[test]
fn an_idle_on_demand_listener_on_d3d12_makes_no_surface_and_holds_no_buffer() {
    let harness = start();
    present_listener(&harness, FramePresent::ON_DEMAND_VERSION, AmmlBand::NORMAL, on_present_idle);
    let window = D3d12Window::new();
    window.submit();
    window.present();
    let counted = OverlayService::surface_counts();

    window.present();
    window.present();
    window.resize(128, 96).unwrap();

    assert_eq!(counts_since(counted), SurfaceCounts::default());
    let idle = format!("idle {}", on_demand_surface(1));
    assert_eq!(lines("idle"), vec![idle.clone(), idle.clone(), idle]);
    assert_eq!(lines("resized"), vec!["resized ok=1 128x96".to_string()]);
}

#[test]
fn an_asking_listener_gets_one_surface_and_later_listeners_see_it() {
    let harness = start();
    present_listener(&harness, FramePresent::ON_DEMAND_VERSION, AmmlBand::NORMAL, on_present_asking);
    present_listener(&harness, FramePresent::ON_DEMAND_VERSION, AmmlBand::LAST, on_present_copying);
    let window = SwapWindow::new();
    let (device, context) = game_context(&window.chain);
    // SAFETY: the context is live.
    unsafe { context.RSSetViewports(Some(&[GAME_VIEWPORT])) };
    let counted = OverlayService::surface_counts();

    window.present();

    assert_eq!(counts_since(counted), SurfaceCounts { made: 1, asked: 1, state_saves: 1 });
    let ok = AmmlStatus::OK.0;
    assert_eq!(
        lines("asking"),
        vec![format!(
            "asking before=[{}] status={ok} made=[{}] again={ok} same_target=true",
            on_demand_surface(0),
            full_surface(0)
        )]
    );
    assert_eq!(lines("copy"), vec![format!("copy status={ok} {}", full_surface(0))]);
    let raw = std::ptr::with_exposed_provenance_mut::<c_void>(FILLED.swap(0, Ordering::SeqCst));
    // SAFETY: the listener stored the texture with one reference, which this takes over.
    let filled = unsafe { ID3D11Texture2D::from_raw(raw) };
    assert_eq!(first_pixel(&device, &filled), [255, 0, 0, 255]);
    assert_eq!(viewport(&context), GAME_VIEWPORT);
}

#[test]
fn an_asking_listener_on_d3d12_draws_and_leaves_no_buffer_held() {
    let harness = start();
    present_listener(&harness, FramePresent::ON_DEMAND_VERSION, AmmlBand::NORMAL, on_present_asking);
    let window = D3d12Window::new();
    window.submit();
    window.present();
    let counted = OverlayService::surface_counts();

    window.present();
    window.resize(128, 96).unwrap();

    assert_eq!(counts_since(counted), SurfaceCounts { made: 1, asked: 1, state_saves: 1 });
    let asking = lines("asking");
    assert!(asking.last().unwrap().contains(&format!("made=[{}]", full_surface(1))), "{asking:?}");
    assert_eq!(lines("resized"), vec!["resized ok=1 128x96".to_string()]);
}

#[test]
fn copy_back_buffer_makes_the_surface_on_demand() {
    let harness = start();
    present_listener(&harness, FramePresent::ON_DEMAND_VERSION, AmmlBand::NORMAL, on_present_copying);
    let window = SwapWindow::new();
    let counted = OverlayService::surface_counts();

    window.present();

    assert_eq!(counts_since(counted), SurfaceCounts { made: 1, asked: 1, state_saves: 1 });
    assert_eq!(lines("copy"), vec![format!("copy status={} {}", AmmlStatus::OK.0, on_demand_surface(0))]);
    let raw = std::ptr::with_exposed_provenance_mut::<c_void>(FILLED.swap(0, Ordering::SeqCst));
    // SAFETY: as above.
    drop(unsafe { ID3D11Texture2D::from_raw(raw) });
}

#[test]
fn an_older_listener_keeps_the_surface_made_before_listeners() {
    let harness = start();
    present_listener(&harness, FramePresent::ON_DEMAND_VERSION, AmmlBand::NORMAL, on_present_idle);
    present_listener(&harness, 1, AmmlBand::LAST, on_present_idle);
    let window = SwapWindow::new();
    let counted = OverlayService::surface_counts();

    window.present();
    window.present();

    assert_eq!(counts_since(counted), SurfaceCounts { made: 2, asked: 0, state_saves: 2 });
    let idle = format!("idle {}", full_surface(0));
    assert_eq!(lines("idle"), vec![idle.clone(), idle.clone(), idle.clone(), idle]);
}

#[test]
fn prepare_surface_outside_frame_present_is_refused() {
    let _harness = start();

    assert_eq!(prepare().0, AmmlStatus::REFUSED);
}

#[test]
fn an_asking_listener_gets_one_surface_in_every_frame() {
    let harness = start();
    present_listener(&harness, FramePresent::ON_DEMAND_VERSION, AmmlBand::NORMAL, on_present_asking);
    let window = SwapWindow::new();
    let counted = OverlayService::surface_counts();

    window.present();
    window.present();
    window.present();

    assert_eq!(counts_since(counted), SurfaceCounts { made: 3, asked: 3, state_saves: 3 });
    let asking = lines("asking");
    assert_eq!(asking.len(), 3);
    let before = format!("before=[{}]", on_demand_surface(0));
    assert!(asking.iter().all(|line| line.contains(&before)), "{asking:?}");
}

#[test]
fn prepare_surface_in_a_frame_made_before_listeners_gives_that_surface() {
    let harness = start();
    present_listener(&harness, 1, AmmlBand::NORMAL, on_present_idle);
    present_listener(&harness, FramePresent::ON_DEMAND_VERSION, AmmlBand::LAST, on_present_asking);
    let window = SwapWindow::new();
    let counted = OverlayService::surface_counts();

    window.present();

    assert_eq!(counts_since(counted), SurfaceCounts { made: 1, asked: 0, state_saves: 1 });
    let ok = AmmlStatus::OK.0;
    assert_eq!(
        lines("asking"),
        vec![format!(
            "asking before=[{}] status={ok} made=[{}] again={ok} same_target=true",
            full_surface(0),
            full_surface(0)
        )]
    );
}

#[test]
fn prepare_surface_in_a_frame_with_no_device_is_not_ready() {
    let harness = start();
    present_listener(&harness, FramePresent::ON_DEMAND_VERSION, AmmlBand::NORMAL, on_present_asking);
    let window = D3d12Window::new();

    // The first Present has no queue yet, so no device and no surface.
    window.present();

    let asking = lines("asking");
    assert_eq!(asking.len(), 1, "{asking:?}");
    assert!(asking[0].contains(&format!("status={} ", AmmlStatus::NOT_READY.0)), "{asking:?}");
    assert_eq!(OverlayService::surface_counts().made, 0);
}

#[test]
fn a_d3d12_game_can_rebuild_its_chain_after_an_asking_listener_drew() {
    let harness = start();
    present_listener(&harness, FramePresent::ON_DEMAND_VERSION, AmmlBand::NORMAL, on_present_asking);
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

    let last = lines("asking").pop().unwrap();
    assert!(last.contains(&format!("status={} ", AmmlStatus::OK.0)), "{last}");
    assert!(last.contains("made=[64x64 wrapped=1 epoch=2 "), "{last}");
}
