use std::ffi::c_void;
use std::panic::AssertUnwindSafe;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock, PoisonError};
use std::time::{Duration, Instant};

use amml_abi::{
    AmmlBand, AmmlEvent, AmmlPhase, AmmlThread, FramePresent, FramePresented, RenderDeviceReset, SurfaceLost,
    SwapChainResize, SwapChainResized,
};
use amml_runtime::{CapturePoint, ResourceKind};
use windows::Win32::Graphics::Direct3D12::{D3D12_COMMAND_LIST_TYPE_DIRECT, ID3D12CommandQueue};
use windows::Win32::Graphics::Dxgi::{DXGI_ERROR_INVALID_CALL, DXGI_PRESENT_TEST, IDXGISwapChain};
use windows::core::{HRESULT, Interface};

use super::{BackBufferCopy, CaptureJob, DrawSurface, DrawWindow, FrameFlow, QueueSlot, StateBackup, SurfaceEnd};
use crate::{EventHub, LoaderState, TextureAdapter, WorldRenderAdapter};

static SHARED: OverlayShared = OverlayShared::new();

/// The process-wide state the DXGI detours reach: a detour has no context pointer, and the vtable it hooks is shared
/// by every chain of the process, so there is one overlay per process. The render side (`FrameFlow`) sits behind a
/// lock that the render thread takes in a Present or ResizeBuffers detour; events are raised with the lock released,
/// so a listener may call into DXGI again.
///
/// Every detour part runs inside `contain`: a panic in loader code is logged once, switches the overlay off, and the
/// detour still calls the original exactly once.
///
/// A frame with `FramePresent` listeners and a surface: the D3D12 back buffer is acquired, the pipeline state saved
/// and the copy window opened before the first listener; all three are drop guards, undone after the last listener
/// (also when a panic unwinds), before the original Present. The surface fields are written back after each
/// listener. `SurfaceLost` is raised before the loader drops anything, `RenderDeviceReset` after a new device is made.
///
/// Captures (`CaptureService`): each frame of the bound chain first reads the copies in flight that are ready. A
/// frame with a surface then copies the `GameOnly` jobs before the first listener and the `WithOverlays` jobs after
/// the `NORMAL` band (the render targets are unbound for that copy); a frame with no `FramePresent` listener copies
/// both in one. A frame with a capture to copy has a surface also when no mod draws. The copies dropped with the
/// device fail their jobs, and so do screenshot requests that found no frame with a surface in time
/// (`CaptureService::expire_shots`).
pub struct OverlayShared {
    state: OnceLock<&'static LoaderState>,
    // Set once every required hook is in; until then the detours only call the original.
    armed: AtomicBool,
    // Set after a panic in a detour part; the detours then only call the original.
    disabled: AtomicBool,
    queue: QueueSlot,
    // The outermost presents of the bound chain so far; the last one is the frame id of the present.
    presents: AtomicU64,
    // Set while `FramePresent` listeners run with a surface lent to them.
    drawing: AtomicBool,
    // A resize inside `FramePresent` was logged.
    nested_resize_logged: AtomicBool,
    flow: Mutex<FrameFlow>,
}

impl OverlayShared {
    // Resource kinds bound to the render device, taken at `SurfaceLost`. None exist yet; capture stream slots are CPU
    // memory and are not among them.
    const RENDER_KINDS: [ResourceKind; 0] = [];

    const fn new() -> Self {
        Self {
            state: OnceLock::new(),
            armed: AtomicBool::new(false),
            disabled: AtomicBool::new(false),
            queue: QueueSlot::new(),
            presents: AtomicU64::new(0),
            drawing: AtomicBool::new(false),
            nested_resize_logged: AtomicBool::new(false),
            flow: Mutex::new(FrameFlow::new()),
        }
    }

    pub fn global() -> &'static Self {
        &SHARED
    }

    /// Takes the one overlay of the process for `state`. False when it was taken already.
    pub fn claim(&self, state: &'static LoaderState) -> bool {
        self.state.set(state).is_ok()
    }

    /// The hooks are in: the detours start to work.
    pub fn arm(&self) {
        self.armed.store(true, Ordering::Release);
    }

    pub fn present_count(&self) -> u64 {
        self.presents.load(Ordering::Acquire)
    }

    /// True while the detours work and a swap chain is bound.
    pub fn is_bound(&self) -> bool {
        self.ready().is_some() && self.lock_flow().targets().is_bound()
    }

    /// The `ExecuteCommandLists` queue kept for the D3D12 path.
    pub fn queue(&self) -> &QueueSlot {
        &self.queue
    }

    /// Before the original Present of `chain`. For the outermost, non-test present of the bound chain: raises
    /// `SurfaceLost` when the surface ended, `RenderDeviceReset` when a new device was made, and `FramePresent` with
    /// the frame's surface; gives the frame id.
    pub fn before_present(&self, chain: *mut c_void, sync: u32, flags: u32, outermost: bool) -> Option<u64> {
        if !outermost || flags & DXGI_PRESENT_TEST.0 != 0 {
            return None;
        }
        let state = self.ready()?;
        self.contain("Present", None, || {
            // SAFETY: DXGI calls Present with a live swap chain for the call.
            let chain = unsafe { IDXGISwapChain::from_raw_borrowed(&chain) }?;
            if state.shutdown().is_closing() {
                self.lock_flow().close();
                self.settle(state);
                return None;
            }
            let end = self.lock_flow().loss(chain);
            if let Some(end) = end {
                self.end_surface(state, end, |flow| flow.end_surface(&self.queue, end));
            }
            if !self.lock_flow().present(state, &self.queue, chain) {
                return None;
            }
            let frame = self.presents.fetch_add(1, Ordering::AcqRel) + 1;
            let listeners = state.events().has_listeners(FramePresent::ID);
            let capturing = state.capture().has_work();
            let wanted = listeners || capturing || self.lock_flow().capture_in_flight();
            // The device is made, and a new one announced, before the frame's targets exist, so a reset listener
            // finds no back buffer held by the loader.
            let ready = wanted && {
                let mut flow = self.lock_flow();
                flow.ensure_device(&self.queue, chain)
            };
            let reset = self.lock_flow().take_reset();
            if let Some(epoch) = reset {
                let mut event = RenderDeviceReset {
                    header: EventHub::header::<RenderDeviceReset>(AmmlPhase::AFTER),
                    device_epoch: epoch,
                };
                state.events().raise(&mut event, &state.events().context_on(AmmlThread::RENDER));
            }
            let reads = self.lock_flow().poll_captures();
            for read in reads {
                state.capture().deliver(read);
            }
            let drawn = ready && (listeners || capturing);
            let surface = if drawn { self.lock_flow().surface(state, chain) } else { None };
            self.draw(state, frame, sync, flags, surface, listeners);
            if capturing {
                // A frame with a surface took every waiting screenshot above; these are requests that found none.
                state.capture().expire_shots(Instant::now());
            }
            self.settle(state);
            Some(frame)
        })
    }

    /// After the original Present of frame `frame`, which gave `result` and took `inside`: notes a lost device and
    /// raises `FramePresented`.
    pub fn after_present(&self, frame: u64, result: HRESULT, inside: Duration) {
        let Some(state) = self.ready() else {
            return;
        };
        self.contain("Present", (), || {
            let lost = self.lock_flow().is_lost_result(result);
            if lost {
                self.end_surface(state, SurfaceEnd::NewDevice, |flow| flow.device_lost(&self.queue, result));
            }
            let mut event = FramePresented {
                header: EventHub::header::<FramePresented>(AmmlPhase::AFTER),
                frame_id: frame,
                result: result.0,
                cpu_us: u32::try_from(inside.as_micros()).unwrap_or(u32::MAX),
            };
            state.events().raise(&mut event, &state.events().context_on(AmmlThread::RENDER));
        });
    }

    /// Before the original `ResizeBuffers` of `chain` with `count` buffers of `width` x `height`. For the bound
    /// chain: every back buffer reference is dropped, `SwapChainResize` is raised, and the size to pass on is given
    /// (a side a listener wrote as 0 keeps what the game asked). `None` for any other chain and while the loader
    /// shuts down; the original then gets the game's arguments.
    pub fn before_resize(&self, chain: *mut c_void, count: u32, width: u32, height: u32) -> Option<(u32, u32)> {
        let state = self.ready()?;
        self.contain("ResizeBuffers", None, || {
            // SAFETY: DXGI calls ResizeBuffers with a live swap chain for the call.
            let chain = unsafe { IDXGISwapChain::from_raw_borrowed(&chain) }?;
            if !self.lock_flow().is_bound_chain(chain) {
                return None;
            }
            if self.drawing.load(Ordering::Acquire) {
                // The frame's surface is lent to the listeners, so its back buffer stays referenced and DXGI refuses
                // the resize; nothing is released under the listeners.
                if !self.nested_resize_logged.swap(true, Ordering::AcqRel) {
                    curia::warn!(
                        "swap chain resize inside FramePresent, refused while the frame holds the back buffer"
                    );
                }
            } else {
                self.lock_flow().before_resize();
            }
            if state.shutdown().is_closing() {
                return None;
            }
            let mut event = SwapChainResize {
                header: EventHub::header::<SwapChainResize>(AmmlPhase::BEFORE),
                width,
                height,
                buffer_count: count,
            };
            state.events().raise(&mut event, &state.events().context_on(AmmlThread::RENDER));
            let given = (Self::side(event.width, width), Self::side(event.height, height));
            if given != (width, height) {
                curia::info!("swap chain resize takes a listener's size", {
                    asked_width: width,
                    asked_height: height,
                    width: given.0,
                    height: given.1
                });
            }
            Some(given)
        })
    }

    /// After the original `ResizeBuffers` of the bound chain, which gave `result`: the window service gets the size
    /// the chain has now, and `SwapChainResized` is raised.
    pub fn after_resize(&self, chain: *mut c_void, result: HRESULT) {
        let Some(state) = self.ready() else {
            return;
        };
        self.contain("ResizeBuffers", (), || {
            // SAFETY: as for `before_resize`.
            let Some(chain) = (unsafe { IDXGISwapChain::from_raw_borrowed(&chain) }) else {
                return;
            };
            let drawing = self.drawing.load(Ordering::Acquire);
            let (size, epoch) = {
                let mut flow = self.lock_flow();
                if drawing && result.is_ok() {
                    flow.before_resize();
                }
                (flow.after_resize(state, chain), flow.epoch())
            };
            if result.is_err() {
                curia::warn!("swap chain resize failed", {
                    result: format!("{:#x}", result.0),
                    width: size.width,
                    height: size.height
                });
                if result == DXGI_ERROR_INVALID_CALL && !drawing && epoch != 0 {
                    curia::warn!("a back buffer is held outside the loader (a mod, the game or another overlay)", {
                        device_epoch: epoch
                    });
                }
            }
            let mut event = SwapChainResized {
                header: EventHub::header::<SwapChainResized>(AmmlPhase::AFTER),
                ok: u8::from(result.is_ok()),
                _reserved: [0; 3],
                width: size.width,
                height: size.height,
            };
            state.events().raise(&mut event, &state.events().context_on(AmmlThread::RENDER));
        });
    }

    /// An `ExecuteCommandLists` on `queue`, on any thread: the first DIRECT queue is kept for the D3D12 path.
    pub fn offer_queue(&self, queue: *mut c_void) {
        let Some(state) = self.ready() else {
            return;
        };
        if state.shutdown().is_closing() || !self.queue.is_empty() {
            return;
        }
        self.contain("ExecuteCommandLists", (), || {
            // SAFETY: D3D12 calls ExecuteCommandLists with a live queue for the call.
            let Some(queue) = (unsafe { ID3D12CommandQueue::from_raw_borrowed(&queue) }) else {
                return;
            };
            // SAFETY: the queue is live.
            if unsafe { queue.GetDesc() }.Type == D3D12_COMMAND_LIST_TYPE_DIRECT {
                self.queue.offer(queue);
            }
        });
    }

    // The loader state while the detours work: armed, not switched off.
    fn ready(&self) -> Option<&'static LoaderState> {
        if !self.armed.load(Ordering::Acquire) || self.disabled.load(Ordering::Acquire) {
            return None;
        }
        self.state.get().copied()
    }

    // A written side of 0 keeps the side the game asked.
    fn side(written: u32, asked: u32) -> u32 {
        if written == 0 { asked } else { written }
    }

    // Raises `SurfaceLost` for `end` when a device was lent (its epoch is not 0), takes the resources of the
    // render-bound kinds, then lets `apply` drop the loader's objects, with the flow lock taken again.
    fn end_surface(&self, state: &LoaderState, end: SurfaceEnd, apply: impl FnOnce(&mut FrameFlow)) {
        let epoch = self.lock_flow().epoch();
        if epoch != 0 {
            let mut event = SurfaceLost {
                header: EventHub::header::<SurfaceLost>(AmmlPhase::BEFORE),
                reason: end.reason(),
                _reserved: [0; 7],
                device_epoch: epoch,
            };
            state.events().raise(&mut event, &state.events().context_on(AmmlThread::RENDER));
            for kind in Self::RENDER_KINDS {
                drop(state.resources().take_kind(kind));
            }
            TextureAdapter::with((), TextureAdapter::device_reset);
            WorldRenderAdapter::with((), WorldRenderAdapter::device_reset);
        }
        apply(&mut self.lock_flow());
        self.settle(state);
    }

    // Fails the jobs of the copies dropped with the device.
    fn settle(&self, state: &LoaderState) {
        let dropped = self.lock_flow().take_dropped();
        if !dropped.is_empty() {
            state.capture().fail(dropped);
        }
    }

    // Copies the back buffer of `surface` for `jobs`; the jobs of a copy that was not made fail.
    fn copy(&self, state: &LoaderState, surface: &DrawSurface, jobs: Vec<CaptureJob>) {
        if jobs.is_empty() {
            return;
        }
        let failed = self.lock_flow().copy(surface, jobs);
        if !failed.is_empty() {
            state.capture().fail(failed);
        }
    }

    // Raises `FramePresent` for frame `frame` and takes its captures. With a surface, the guards are set up in this order
    // and undone in the reverse one: the wrapped back buffer is acquired, the `GameOnly` copy made, the state saved,
    // the copy window opened and the draw flag set; the `WithOverlays` copy is made after the `NORMAL` band. With no
    // `FramePresent` listener (`listeners` false) both copies are one and nothing is raised.
    fn draw(
        &self,
        state: &LoaderState,
        frame: u64,
        sync: u32,
        flags: u32,
        surface: Option<DrawSurface>,
        listeners: bool,
    ) {
        let lent = surface.as_ref().map_or_else(DrawSurface::empty, DrawSurface::abi);
        let mut event = FramePresent {
            header: EventHub::header::<FramePresent>(AmmlPhase::BEFORE),
            frame_id: frame,
            sync_interval: sync,
            flags,
            surface: lent,
        };
        let context = state.events().context_on(AmmlThread::RENDER);
        let keep = |event: &mut FramePresent| event.surface = lent;
        let Some(surface) = surface else {
            state.events().raise_keeping(&mut event, &context, &keep);
            return;
        };
        let _acquired = surface.acquire();
        let capture = state.capture();
        let mut game_only = capture.due(CapturePoint::BeforeListeners, frame);
        if !listeners {
            game_only.extend(capture.due(CapturePoint::AfterNormalBand, frame));
            self.copy(state, &surface, game_only);
            return;
        }
        self.copy(state, &surface, game_only);
        let _state = StateBackup::capture(&surface.context);
        let _copy = BackBufferCopy::open(&surface);
        let _drawing = DrawWindow::open(&self.drawing);
        state.events().raise_split_keeping(&mut event, &context, &keep, AmmlBand::NORMAL, |_| {
            let with_overlays = capture.due(CapturePoint::AfterNormalBand, frame);
            if !with_overlays.is_empty() {
                // SAFETY: the context is the frame's, on the render thread; binding no target takes no object, and
                // the state backup puts the listeners' targets back after the last listener.
                unsafe { surface.context.OMSetRenderTargets(None, None) };
                self.copy(state, &surface, with_overlays);
            }
        });
    }

    // Runs `body` for detour `entry` and gives its value. A panic is logged once, switches the overlay off, drops
    // every back buffer reference, and gives `fallback`, so the detour still calls the original.
    fn contain<T>(&self, entry: &'static str, fallback: T, body: impl FnOnce() -> T) -> T {
        match std::panic::catch_unwind(AssertUnwindSafe(body)) {
            Ok(value) => value,
            Err(_) => {
                if !self.disabled.swap(true, Ordering::AcqRel) {
                    curia::error!("overlay detour panicked, the overlay is off for this session", { entry: entry });
                }
                let released = std::panic::catch_unwind(AssertUnwindSafe(|| {
                    self.lock_flow().close();
                    if let Some(state) = self.state.get() {
                        self.settle(state);
                    }
                }));
                if released.is_err() {
                    curia::error!("overlay could not drop its back buffer references after a panic");
                }
                fallback
            }
        }
    }

    // A panic while the lock was held switched the overlay off; the plain data inside is still usable.
    fn lock_flow(&self) -> MutexGuard<'_, FrameFlow> {
        self.flow.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
