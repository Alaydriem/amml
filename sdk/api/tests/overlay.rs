use std::cell::RefCell;
use std::ffi::c_void;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use amml_abi::{AmmlDrawSurface, AmmlStatus, FramePresent};
use amml_api::testing::FakeGame;
use amml_api::{ApiError, Area, CaptureRequest, FramePresentExt, HookOrder, ShotId, ShotLayers, Size};

thread_local! {
    // A `FramePresent` a listener kept past its call.
    static KEPT: RefCell<Option<FramePresent>> = const { RefCell::new(None) };
}

fn fake_pointer(address: usize) -> *mut c_void {
    std::ptr::without_provenance_mut(address)
}

fn surface(device: *mut c_void) -> AmmlDrawSurface {
    AmmlDrawSurface {
        size: AmmlDrawSurface::SIZE_V1,
        version: 1,
        width: 1280,
        height: 720,
        format: 28,
        wrapped: 1,
        _reserved: [0; 3],
        device_epoch: 3,
        device,
        context: fake_pointer(0x2000),
        target: fake_pointer(0x3000),
        back_buffer: fake_pointer(0x4000),
    }
}

fn present(surface: AmmlDrawSurface) -> FramePresent {
    FramePresent { header: FakeGame::header::<FramePresent>(), frame_id: 7, sync_interval: 1, flags: 0, surface }
}

fn request(ring: u8, every_nth_present: u32) -> CaptureRequest {
    CaptureRequest { layers: ShotLayers::WithOverlays, every_nth_present, ring, max_size: None }
}

// A fake with the capture area offered and the swap chain bound (the open gate).
fn capturing() -> FakeGame {
    let fake = FakeGame::new();
    fake.offer_area(Area::OVERLAY_CAPTURE);
    fake.open_gate();
    fake
}

#[test]
fn a_dropped_capture_frame_gives_its_slot_back() {
    let fake = capturing();
    let mut stream = fake.ctx().overlay().open_capture_stream(request(1, 1)).unwrap();
    let bgra = vec![1, 2, 3, 255, 4, 5, 6, 255];
    fake.push_capture_frame(2, 1, bgra.clone());

    let frame = stream.take().unwrap();
    assert_eq!(frame.pixels(), bgra.as_slice());
    assert_eq!(frame.size(), Size { width: 2, height: 1 });
    assert_eq!(frame.stride(), 8);
    assert_eq!(frame.present_index(), 1);
    drop(frame);
    assert_eq!(stream.free_slots(), 1);
    assert!(stream.take().is_none());
}

#[test]
fn a_taken_frame_holds_its_slot() {
    let fake = capturing();
    let mut stream = fake.ctx().overlay().open_capture_stream(request(1, 1)).unwrap();
    fake.push_capture_frame(1, 1, vec![0, 0, 0, 255]);
    assert_eq!(stream.free_slots(), 0);

    let frame = stream.take().unwrap();
    // The slot of a taken frame is not free: this present is skipped.
    fake.push_capture_frame(1, 1, vec![9, 9, 9, 255]);
    drop(frame);

    assert_eq!(fake.capture_skipped(), 1);
    assert!(stream.take().is_none());
}

#[test]
fn a_dropped_stream_closes() {
    let fake = capturing();
    let stream = fake.ctx().overlay().open_capture_stream(request(2, 1)).unwrap();
    assert_eq!(fake.open_capture_streams(), 1);

    drop(stream);

    assert_eq!(fake.open_capture_streams(), 0);
}

#[test]
fn a_full_ring_skips_and_the_present_index_shows_the_gap() {
    let fake = capturing();
    let mut stream = fake.ctx().overlay().open_capture_stream(request(1, 1)).unwrap();
    for _ in 0..3 {
        fake.push_capture_frame(1, 1, vec![0, 0, 0, 255]);
    }
    let first = stream.take().map(|frame| frame.present_index());
    fake.push_capture_frame(1, 1, vec![0, 0, 0, 255]);
    let next = stream.take().map(|frame| frame.present_index());

    assert_eq!((first, next), (Some(1), Some(4)));
    assert_eq!(fake.capture_skipped(), 2);
}

#[test]
fn a_stream_takes_every_nth_present_only() {
    let fake = capturing();
    let mut stream = fake.ctx().overlay().open_capture_stream(request(4, 2)).unwrap();
    for _ in 0..5 {
        fake.push_capture_frame(1, 1, vec![0, 0, 0, 255]);
    }
    let mut indices = Vec::new();
    while let Some(frame) = stream.take() {
        indices.push(frame.present_index());
    }

    assert_eq!(indices, [2, 4]);
    assert_eq!(fake.capture_skipped(), 0);
}

#[test]
fn a_frame_shorter_than_its_rows_is_given_back_and_not_lent() {
    let fake = capturing();
    let mut stream = fake.ctx().overlay().open_capture_stream(request(1, 1)).unwrap();
    // Two pixels named, one pixel of bytes.
    fake.push_capture_frame(2, 1, vec![0, 0, 0, 255]);

    assert!(stream.take().is_none());
    assert_eq!(stream.free_slots(), 1, "the refused frame went back to the loader");
}

#[test]
fn a_capture_stream_needs_its_area_a_bound_chain_and_a_valid_ring() {
    let fake = FakeGame::new();
    let overlay = fake.ctx().overlay();
    assert_eq!(overlay.open_capture_stream(request(1, 1)).err(), Some(ApiError::NotAvailable));

    fake.offer_area(Area::OVERLAY_CAPTURE);
    assert_eq!(overlay.open_capture_stream(request(1, 1)).err(), Some(ApiError::NotReady));
    assert_eq!(overlay.open_capture_stream(request(0, 1)).err(), Some(ApiError::Loader(AmmlStatus::INVALID_ARGUMENT)));

    fake.open_gate();
    assert_eq!(overlay.open_capture_stream(request(9, 1)).err(), Some(ApiError::Loader(AmmlStatus::INVALID_ARGUMENT)));
    assert_eq!(fake.open_capture_streams(), 0);
}

#[test]
fn a_loader_without_an_overlay_table_serves_no_overlay() {
    let fake = FakeGame::with_v2_services();
    fake.offer_area(Area::OVERLAY_CAPTURE);
    fake.offer_area(Area::OVERLAY_SCREENSHOT);
    fake.offer_area(Area::OVERLAY_SURFACE);
    fake.open_gate();
    fake.push_capture_frame(1, 1, vec![0, 0, 0, 255]);
    let overlay = fake.ctx().overlay();

    assert_eq!(overlay.open_capture_stream(request(1, 1)).err(), Some(ApiError::NotAvailable));
    assert_eq!(overlay.screenshot(Path::new("C:\\shots\\a.png"), ShotLayers::GameOnly), Err(ApiError::NotAvailable));
    assert_eq!(overlay.present_count(), 0);
    assert!(!fake.ctx().supports(Area::OVERLAY_SURFACE));
}

#[test]
fn a_screenshot_needs_an_absolute_path_and_gives_its_id() {
    let fake = FakeGame::new();
    fake.offer_area(Area::OVERLAY_SCREENSHOT);
    let overlay = fake.ctx().overlay();
    let path = Path::new("C:\\shots\\game.png");
    assert_eq!(overlay.screenshot(path, ShotLayers::GameOnly), Err(ApiError::NotReady));

    fake.open_gate();
    let first = overlay.screenshot(path, ShotLayers::GameOnly).unwrap();
    let second = overlay.screenshot(Path::new("C:\\shots\\drawn.png"), ShotLayers::WithOverlays).unwrap();
    let relative = overlay.screenshot(Path::new("shots\\game.png"), ShotLayers::GameOnly);

    assert_ne!(first, second);
    assert_ne!(first, ShotId(0));
    assert_eq!(relative, Err(ApiError::Loader(AmmlStatus::INVALID_ARGUMENT)));
    assert_eq!(
        fake.screenshots(),
        [
            (PathBuf::from("C:\\shots\\game.png"), ShotLayers::GameOnly),
            (PathBuf::from("C:\\shots\\drawn.png"), ShotLayers::WithOverlays),
        ]
    );
}

#[test]
fn the_present_count_follows_the_presents_and_needs_the_surface_area() {
    let fake = FakeGame::new();
    let overlay = fake.ctx().overlay();
    fake.push_capture_frame(1, 1, vec![0, 0, 0, 255]);
    fake.push_capture_frame(1, 1, vec![0, 0, 0, 255]);
    assert_eq!(overlay.present_count(), 0);

    fake.offer_area(Area::OVERLAY_SURFACE);

    assert_eq!(overlay.present_count(), 2);
}

#[test]
fn an_empty_surface_is_none() {
    let fake = FakeGame::new();
    fake.offer::<FramePresent>();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<FramePresent>(HookOrder::NORMAL, move |event, _| {
            record.lock().unwrap().push(event.surface().is_none());
        })
        .unwrap();

    fake.raise(&mut present(surface(std::ptr::null_mut())));
    let mut short = surface(fake_pointer(0x1000));
    short.size = AmmlDrawSurface::SIZE_V1 - 8;
    fake.raise(&mut present(short));

    assert_eq!(*seen.lock().unwrap(), [true, true]);
}

#[test]
fn a_surface_is_lent_to_its_listener_call_only() {
    let fake = FakeGame::new();
    fake.offer::<FramePresent>();
    let device = fake_pointer(0x1000);
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<FramePresent>(HookOrder::NORMAL, move |event, _| {
            let lent = event.surface().map(|surface| {
                (
                    surface.size(),
                    surface.format(),
                    surface.wrapped(),
                    surface.device_epoch(),
                    surface.device().as_raw().addr(),
                    surface.context().as_raw().addr(),
                    surface.target().as_raw().addr(),
                )
            });
            // A listener's write to the surface fields does not change the lent surface.
            event.surface.device = std::ptr::null_mut();
            event.surface.target = fake_pointer(0x9000);
            let after_write =
                event.surface().map(|surface| (surface.device().as_raw().addr(), surface.target().as_raw().addr()));
            // A copy of the event is not the event of the call.
            let copy = *event;
            let copied = copy.surface().is_some();
            KEPT.with(|kept| *kept.borrow_mut() = Some(*event));
            record.lock().unwrap().push((lent, after_write, copied));
        })
        .unwrap();

    fake.raise(&mut present(surface(device)));

    let size = Size { width: 1280, height: 720 };
    assert_eq!(
        *seen.lock().unwrap(),
        [(Some((size, 28, true, 3, 0x1000, 0x2000, 0x3000)), Some((0x1000, 0x3000)), false)]
    );
    // Past its call, the kept event gives no surface.
    assert!(KEPT.with(|kept| kept.borrow().as_ref().is_some_and(|event| event.surface().is_none())));
}

// A fake that makes the surface on demand: the surface area and `FramePresent` at the on-demand version.
fn on_demand() -> FakeGame {
    let fake = FakeGame::new();
    fake.offer::<FramePresent>();
    fake.offer_area(Area::OVERLAY_SURFACE);
    fake.offer_surface_on_demand();
    fake
}

// The surface a loader lends before a listener asks: a device, the size and format, no other object.
fn waiting(device: *mut c_void) -> AmmlDrawSurface {
    AmmlDrawSurface {
        context: std::ptr::null_mut(),
        target: std::ptr::null_mut(),
        back_buffer: std::ptr::null_mut(),
        ..surface(device)
    }
}

#[test]
fn an_idle_listener_never_asks_for_the_surface() {
    let fake = on_demand();
    fake.set_prepared_surface(surface(fake_pointer(0x1000)));
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<FramePresent>(HookOrder::NORMAL, move |event, _| {
            let lent = event.surface().map(|surface| {
                (surface.size(), surface.format(), surface.device_epoch(), surface.device().as_raw().addr())
            });
            record.lock().unwrap().push(lent);
        })
        .unwrap();

    fake.raise(&mut present(waiting(fake_pointer(0x1000))));

    assert_eq!(*seen.lock().unwrap(), [Some((Size { width: 1280, height: 720 }, 28, 3, 0x1000))]);
    assert_eq!(fake.prepare_surface_calls(), 0);
}

#[test]
fn a_drawing_listener_asks_once_and_gets_the_made_surface() {
    let fake = on_demand();
    fake.set_prepared_surface(surface(fake_pointer(0x1000)));
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<FramePresent>(HookOrder::NORMAL, move |event, _| {
            if let Some(surface) = event.surface() {
                let ready = surface.ready();
                record.lock().unwrap().push((
                    ready,
                    surface.target().as_raw().addr(),
                    surface.context().as_raw().addr(),
                ));
            }
        })
        .unwrap();

    fake.raise(&mut present(waiting(fake_pointer(0x1000))));

    assert_eq!(*seen.lock().unwrap(), [(true, 0x3000, 0x2000)]);
    assert_eq!(fake.prepare_surface_calls(), 1);
}

#[test]
fn a_surface_the_loader_cannot_make_gives_absent_handles_and_is_asked_once() {
    let fake = on_demand();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<FramePresent>(HookOrder::NORMAL, move |event, _| {
            if let Some(surface) = event.surface() {
                record.lock().unwrap().push((
                    surface.ready(),
                    surface.target().as_raw().is_null(),
                    surface.context().as_raw().is_null(),
                ));
            }
        })
        .unwrap();

    fake.raise(&mut present(waiting(fake_pointer(0x1000))));

    assert_eq!(*seen.lock().unwrap(), [(false, true, true)]);
    assert_eq!(fake.prepare_surface_calls(), 1);
}

#[test]
fn without_the_on_demand_version_a_surface_with_no_back_buffer_is_none() {
    let fake = FakeGame::new();
    fake.offer::<FramePresent>();
    fake.offer_area(Area::OVERLAY_SURFACE);
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<FramePresent>(HookOrder::NORMAL, move |event, _| {
            record.lock().unwrap().push(event.surface().is_none());
        })
        .unwrap();

    fake.raise(&mut present(waiting(fake_pointer(0x1000))));

    assert_eq!(*seen.lock().unwrap(), [true]);
    assert_eq!(fake.prepare_surface_calls(), 0);
}

#[cfg(feature = "d3d")]
mod d3d {
    use std::sync::{Arc, Mutex};

    use amml_abi::{AmmlDrawSurface, FramePresent};
    use amml_api::testing::FakeGame;
    use amml_api::{ApiError, Area, ComRef, FramePresentExt, HookOrder};
    use windows::Win32::Foundation::HMODULE;
    use windows::Win32::Graphics::Direct3D::{D3D_DRIVER_TYPE_WARP, D3D_FEATURE_LEVEL};
    use windows::Win32::Graphics::Direct3D11::{
        D3D11_BIND_RENDER_TARGET, D3D11_CREATE_DEVICE_FLAG, D3D11_SDK_VERSION, D3D11_TEXTURE2D_DESC,
        D3D11_USAGE_DEFAULT, D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext, ID3D11RenderTargetView,
        ID3D11Texture2D,
    };
    use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC};
    use windows::core::Interface;

    struct Warp {
        device: ID3D11Device,
        context: ID3D11DeviceContext,
        texture: ID3D11Texture2D,
        target: ID3D11RenderTargetView,
    }

    impl Warp {
        fn new() -> Self {
            let mut device = None;
            let mut context = None;
            // SAFETY: every out pointer is a live local; WARP needs no adapter.
            unsafe {
                D3D11CreateDevice(
                    None,
                    D3D_DRIVER_TYPE_WARP,
                    HMODULE::default(),
                    D3D11_CREATE_DEVICE_FLAG(0),
                    None,
                    D3D11_SDK_VERSION,
                    Some(&mut device),
                    None,
                    Some(&mut context),
                )
            }
            .unwrap();
            let (device, context) = (device.unwrap(), context.unwrap());
            let desc = D3D11_TEXTURE2D_DESC {
                Width: 64,
                Height: 64,
                MipLevels: 1,
                ArraySize: 1,
                Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                Usage: D3D11_USAGE_DEFAULT,
                BindFlags: D3D11_BIND_RENDER_TARGET.0 as u32,
                CPUAccessFlags: 0,
                MiscFlags: 0,
            };
            let mut texture = None;
            // SAFETY: `desc` and the out pointer are live locals.
            unsafe { device.CreateTexture2D(&desc, None, Some(&mut texture)) }.unwrap();
            let texture = texture.unwrap();
            let mut target = None;
            // SAFETY: `texture` is a render target texture of `device`; the out pointer is a live local.
            unsafe { device.CreateRenderTargetView(&texture, None, Some(&mut target)) }.unwrap();
            Self { device, context, texture, target: target.unwrap() }
        }

        fn surface(&self) -> AmmlDrawSurface {
            AmmlDrawSurface {
                size: AmmlDrawSurface::SIZE_V1,
                version: 1,
                width: 64,
                height: 64,
                format: DXGI_FORMAT_B8G8R8A8_UNORM.0 as u32,
                wrapped: 0,
                _reserved: [0; 3],
                device_epoch: 1,
                device: self.device.as_raw(),
                context: self.context.as_raw(),
                target: self.target.as_raw(),
                back_buffer: self.texture.as_raw(),
            }
        }
    }

    #[test]
    fn with_lends_the_surface_objects_as_d3d11_types() {
        let fake = FakeGame::new();
        fake.offer::<FramePresent>();
        fake.offer_area(Area::OVERLAY_SURFACE);
        let warp = Warp::new();
        let destination = warp.texture.clone();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let record = seen.clone();
        let dest = send_texture(destination);
        let _listener = fake
            .ctx()
            .events()
            .listen::<FramePresent>(HookOrder::NORMAL, move |event, _| {
                let Some(surface) = event.surface() else {
                    record.lock().unwrap().push(None);
                    return;
                };
                // SAFETY: the device is live for the call.
                let level = surface.device().with(|device| unsafe { device.GetFeatureLevel() });
                surface.context().with(|context| {
                    surface.target().with(|target| {
                        // SAFETY: the target view and the context belong to one live device.
                        unsafe { context.ClearRenderTargetView(target, &[1.0, 0.0, 1.0, 1.0]) };
                    })
                });
                let copied = surface.copy_back_buffer(ComRef::from_interface(&dest.0));
                record.lock().unwrap().push(Some((level, copied)));
            })
            .unwrap();

        fake.raise(&mut FramePresent {
            header: FakeGame::header::<FramePresent>(),
            frame_id: 1,
            sync_interval: 1,
            flags: 0,
            surface: warp.surface(),
        });

        // SAFETY: the device is live.
        let level: D3D_FEATURE_LEVEL = unsafe { warp.device.GetFeatureLevel() };
        // The fake has no back buffer to copy, so it refuses every copy.
        assert_eq!(*seen.lock().unwrap(), [Some((Some(level), Err(ApiError::Refused)))]);
    }

    // A texture moved into a listener closure, which must be `Send`; the test calls it on one thread.
    struct SendTexture(ID3D11Texture2D);

    // SAFETY: the test raises the event on the thread that made the texture.
    #[allow(unsafe_code)]
    unsafe impl Send for SendTexture {}

    fn send_texture(texture: ID3D11Texture2D) -> SendTexture {
        SendTexture(texture)
    }
}
