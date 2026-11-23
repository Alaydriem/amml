use amml_abi::{AmmlDrawSurface, AmmlSize, AmmlSurfaceLoss};
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Direct3D11::ID3D11Device;
use windows::Win32::Graphics::Direct3D12::ID3D12Device;
use windows::Win32::Graphics::Dxgi::{
    DXGI_ERROR_DEVICE_REMOVED, DXGI_ERROR_DEVICE_RESET, DXGI_SWAP_CHAIN_DESC, IDXGISwapChain,
};
use windows::Win32::UI::WindowsAndMessaging::{InternalGetWindowText, IsWindow};
use windows::core::{HRESULT, IUnknown, Interface};

use super::{
    CaptureJob, CaptureRead, ChainTargets, DeviceStart, DrawSurface, FrameCapture, OverlayDevice, QueueSlot, SurfaceEnd,
};
use crate::{LoaderState, WindowService};

/// The render side of the overlay: which chain and window it is bound to, the device listeners draw with, and the
/// decisions of one Present or ResizeBuffers. Lives in `OverlayShared` behind its lock, which no event raise holds.
///
/// Binding: nothing is bound before the game gate opens. The first outermost Present after that binds its chain,
/// and its window is attached to the loader's window service. A chain on another window is ignored while the bound
/// window exists; once it is gone, the next chain presented is bound. A new chain or a new device on the bound
/// window ends the surface and binds again. A chain whose device is removed (`GetDeviceRemovedReason` is not `S_OK`)
/// is not bound; liveness is asked of the device itself, so a new device at the freed address of a removed one binds.
///
/// Device: made at the first frame that has a `FramePresent` listener, and kept until the surface ends. Each device
/// has the next `device_epoch` (from 1); a device after the first is announced with `RenderDeviceReset`. While it
/// lives, its address cannot be taken by a new device, so the identity compare by address stays sound. A frame with a
/// capture to copy (or a copy in flight) also makes the device.
///
/// Captures: the staging textures of the back-buffer copies (`FrameCapture`) are made on the device and go with it,
/// so a surface loss, a lost device and the shutdown drop them; the jobs of the copies then in flight wait in
/// `take_dropped` for the caller to fail them.
#[derive(Default)]
pub struct FrameFlow {
    targets: ChainTargets,
    device: Option<OverlayDevice>,
    // No device can be made on the bound chain; tried again after the next bind.
    device_failed: bool,
    // The epoch of `device`; 0 while there is none.
    epoch: u64,
    // The last epoch given.
    last_epoch: u64,
    // A device made after the first, to be announced with `RenderDeviceReset`.
    reset: Option<u64>,
    queue_wait_logged: bool,
    // A failure to make the frame's targets was logged since the last bind or resize.
    target_warned: bool,
    // The window drawn on, as its handle value; 0 when none is bound.
    bound_window: usize,
    // Identity only: the device of the bound chain (its `IUnknown`); 0 when none is bound.
    bound_device: usize,
    // Identity only: the D3D12 device of the bound chain; 0 for a D3D11 chain. DXGI gives a D3D12 chain's device,
    // not its queue, from `GetDevice`.
    chain_d3d12: usize,
    queue_logged: bool,
    capture: FrameCapture,
    // The jobs of copies dropped with the device, for `take_dropped`.
    dropped: Vec<CaptureJob>,
}

impl FrameFlow {
    // The longest window title the bind log line carries, in UTF-16 units.
    const TITLE_UNITS: usize = 128;

    pub const fn new() -> Self {
        Self {
            targets: ChainTargets::new(),
            device: None,
            device_failed: false,
            epoch: 0,
            last_epoch: 0,
            reset: None,
            queue_wait_logged: false,
            target_warned: false,
            bound_window: 0,
            bound_device: 0,
            chain_d3d12: 0,
            queue_logged: false,
            capture: FrameCapture::new(),
            dropped: Vec::new(),
        }
    }

    /// What ends the surface at this outermost Present of `chain`, if anything. Nothing is dropped here: the caller
    /// raises `SurfaceLost` first, then calls `end_surface`.
    pub fn loss(&self, chain: &IDXGISwapChain) -> Option<SurfaceEnd> {
        if self.bound_window != 0 && !Self::is_window(self.bound_window) {
            return Some(SurfaceEnd::WindowGone);
        }
        if self.targets.is_chain(chain) {
            return (Self::device_of(chain) != self.bound_device).then_some(SurfaceEnd::NewDevice);
        }
        if !self.targets.is_bound() {
            return None;
        }
        // SAFETY: DXGI calls Present with a live swap chain for the call.
        let desc = unsafe { chain.GetDesc() }.ok()?;
        if desc.OutputWindow.0.expose_provenance() != self.bound_window {
            return None;
        }
        let device = Self::device_of(chain);
        if device == 0 || Self::is_removed(chain) {
            return None;
        }
        Some(SurfaceEnd::NewChain { device_changed: device != self.bound_device })
    }

    /// Ends the surface for `end`, after `SurfaceLost` was raised: every back buffer reference and the device go.
    pub fn end_surface(&mut self, queue: &QueueSlot, end: SurfaceEnd) {
        match end {
            SurfaceEnd::WindowGone => {
                curia::info!("overlay window is gone, the next swap chain is bound", { hwnd: self.bound_window });
                self.lose_surface(queue, end.reason());
                self.unbind();
            }
            SurfaceEnd::NewDevice => {
                curia::info!("overlay sees a new device on the bound swap chain");
                self.lose_surface(queue, end.reason());
                self.targets.forget();
            }
            SurfaceEnd::NewChain { device_changed } => {
                curia::info!("overlay sees a new swap chain on the bound window", { new_device: device_changed });
                self.lose_surface(queue, end.reason());
                self.targets.forget();
            }
        }
    }

    /// The outermost Present of `chain`, not a test Present, after any loss was ended: binds the chain when it may.
    /// True when it is the bound chain, so the present is a frame.
    pub fn present(&mut self, state: &LoaderState, queue: &QueueSlot, chain: &IDXGISwapChain) -> bool {
        if self.targets.is_chain(chain) {
            self.note_queue(queue);
            return true;
        }
        if self.bound_window == 0 && !state.events().gate().is_open() {
            return false;
        }
        // SAFETY: DXGI calls Present with a live swap chain for the call.
        let Ok(desc) = (unsafe { chain.GetDesc() }) else {
            return false;
        };
        let window = desc.OutputWindow.0.expose_provenance();
        if self.bound_window != 0 && window != self.bound_window {
            return false;
        }
        let device = Self::device_of(chain);
        if device == 0 || Self::is_removed(chain) {
            return false;
        }
        if self.targets.is_bound() {
            // `loss` did not see this chain (its `GetDesc` failed then); it ends the surface at the next present.
            return false;
        }
        self.bind(state, chain, &desc, window, device);
        true
    }

    /// The epoch of the device listeners draw with; 0 while there is none.
    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    /// True when `result`, of a present of the bound chain, says its device is removed or reset.
    pub fn is_lost_result(&self, result: HRESULT) -> bool {
        (result == DXGI_ERROR_DEVICE_REMOVED || result == DXGI_ERROR_DEVICE_RESET) && self.targets.is_bound()
    }

    /// After a present that gave `result` (device removed or reset) and the `SurfaceLost` it raised: ends the surface
    /// and unbinds the chain. The window stays, and the next chain on it with a new device is bound.
    pub fn device_lost(&mut self, queue: &QueueSlot, result: HRESULT) {
        curia::warn!("overlay device lost", { result: format!("{:#x}", result.0) });
        self.lose_surface(queue, AmmlSurfaceLoss::DEVICE_CHANGED);
        self.bound_device = 0;
        self.targets.forget();
    }

    /// Makes the device listeners draw with when there is none (after a bind or a loss). True when a device exists.
    /// A device after the first is kept for `take_reset`, which the caller raises before the frame's targets exist.
    pub fn ensure_device(&mut self, queue: &QueueSlot, chain: &IDXGISwapChain) -> bool {
        self.device.is_some() || self.start_device(queue, chain)
    }

    /// The draw surface of this frame of the bound `chain`, with a reference of its own to each object, made after
    /// `ensure_device`. `None` without a device, when no target can be made, and while a side of the buffer is 0. On
    /// a D3D11 frame, a buffer size or format other than the kept one is given to the window service.
    pub fn surface(&mut self, state: &LoaderState, chain: &IDXGISwapChain) -> Option<DrawSurface> {
        let device = self.device.as_ref()?;
        let Some(on12) = &device.on12 else {
            let made = ChainTargets::native(&device.device, chain);
            let (back_buffer, desc, target) = match made {
                Ok(made) => made,
                Err(error) => {
                    Self::warn_target(&mut self.target_warned, &error);
                    return None;
                }
            };
            let format = desc.Format.0 as u32;
            if self.targets.note_buffer(desc.Width, desc.Height, format, desc.SampleDesc.Count) {
                state.window().set_buffer(AmmlSize { width: desc.Width, height: desc.Height });
            }
            if desc.Width == 0 || desc.Height == 0 {
                return None;
            }
            return Some(DrawSurface {
                device: device.device.clone(),
                context: device.context.clone(),
                target,
                back_buffer,
                on12: None,
                width: desc.Width,
                height: desc.Height,
                format,
                samples: desc.SampleDesc.Count,
                epoch: self.epoch,
            });
        };
        let (width, height) = self.targets.size();
        if width == 0 || height == 0 {
            return None;
        }
        let (back_buffer, target) = match ChainTargets::wrapped(&device.device, on12, chain) {
            Ok(made) => made,
            Err(error) => {
                Self::warn_target(&mut self.target_warned, &error);
                return None;
            }
        };
        Some(DrawSurface {
            device: device.device.clone(),
            context: device.context.clone(),
            target,
            back_buffer,
            on12: Some(on12.clone()),
            width,
            height,
            format: self.targets.format(),
            samples: self.targets.samples(),
            epoch: self.epoch,
        })
    }

    /// The surface listeners that take it on demand get before one asks: the device, the kept size and format, and no
    /// context, target or back buffer. `None` without a device and while a side of the buffer is 0. On a D3D11 frame
    /// the buffer's description is read (no view is made), and a size or format other than the kept one is given to
    /// the window service, as `surface` does.
    pub fn waiting_surface(&mut self, state: &LoaderState, chain: &IDXGISwapChain) -> Option<AmmlDrawSurface> {
        if !self.device.as_ref()?.is_on12() {
            match ChainTargets::native_desc(chain) {
                Ok(desc) => {
                    let format = desc.Format.0 as u32;
                    if self.targets.note_buffer(desc.Width, desc.Height, format, desc.SampleDesc.Count) {
                        state.window().set_buffer(AmmlSize { width: desc.Width, height: desc.Height });
                    }
                }
                Err(error) => {
                    Self::warn_target(&mut self.target_warned, &error);
                    return None;
                }
            }
        }
        let device = self.device.as_ref()?;
        let (width, height) = self.targets.size();
        if width == 0 || height == 0 {
            return None;
        }
        let format = self.targets.format();
        Some(DrawSurface::waiting(&device.device, device.is_on12(), width, height, format, self.epoch))
    }

    /// The epoch of a device made after the first, once, for `RenderDeviceReset`.
    pub fn take_reset(&mut self) -> Option<u64> {
        self.reset.take()
    }

    /// True when `chain` is the bound chain.
    pub fn is_bound_chain(&self, chain: &IDXGISwapChain) -> bool {
        self.targets.is_chain(chain)
    }

    /// Before `ResizeBuffers` of the bound chain: no back buffer is held between frames, and on the D3D12 path the
    /// context is cleared and flushed, so nothing holds a buffer bound or queued.
    pub fn before_resize(&mut self) {
        if let Some(device) = &self.device {
            device.clear();
        }
    }

    /// After `ResizeBuffers` of the bound chain, whether it succeeded: the size the chain has now (its arguments may
    /// be 0, and a failure keeps the old buffers), given to the window service.
    pub fn after_resize(&mut self, state: &LoaderState, chain: &IDXGISwapChain) -> AmmlSize {
        // SAFETY: the chain is live for the ResizeBuffers that called this.
        if let Ok(desc) = unsafe { chain.GetDesc() } {
            self.targets.set_desc(&desc);
        }
        self.target_warned = false;
        let (width, height) = self.targets.size();
        let size = AmmlSize { width, height };
        state.window().set_buffer(size);
        size
    }

    /// Drops every back buffer reference and the device while the loader shuts down or after a panic, on the render
    /// thread; nothing is made again.
    pub fn close(&mut self) {
        self.drop_device();
    }

    pub fn targets(&self) -> &ChainTargets {
        &self.targets
    }

    /// Copies the back buffer of `surface` into a staging texture for `jobs`; gives back the jobs when no copy was
    /// made. On the D3D12 path the caller holds the wrapped back buffer acquired.
    pub fn copy(&mut self, surface: &DrawSurface, jobs: Vec<CaptureJob>) -> Vec<CaptureJob> {
        match self.capture.request(&surface.device, &surface.context, &surface.back_buffer, jobs) {
            Ok(()) => Vec::new(),
            Err(jobs) => jobs,
        }
    }

    /// Reads the copies in flight that are ready, with no wait; each read or failed copy is given once.
    pub fn poll_captures(&mut self) -> Vec<CaptureRead> {
        match &self.device {
            Some(device) if self.capture.in_flight() => self.capture.poll(&device.context),
            _ => Vec::new(),
        }
    }

    /// True while a copy waits to be read.
    pub fn capture_in_flight(&self) -> bool {
        self.capture.in_flight()
    }

    /// The jobs of the copies dropped with the device since the last call.
    pub fn take_dropped(&mut self) -> Vec<CaptureJob> {
        std::mem::take(&mut self.dropped)
    }

    fn start_device(&mut self, queue: &QueueSlot, chain: &IDXGISwapChain) -> bool {
        if self.device_failed {
            return false;
        }
        match OverlayDevice::start(chain, queue) {
            DeviceStart::Ready(device) => {
                self.last_epoch += 1;
                self.epoch = self.last_epoch;
                if self.epoch > 1 {
                    self.reset = Some(self.epoch);
                }
                curia::info!("overlay device ready", { path: device.path(), device_epoch: self.epoch });
                self.device = Some(device);
                true
            }
            DeviceStart::Waiting => {
                if !self.queue_wait_logged {
                    self.queue_wait_logged = true;
                    curia::info!("overlay is waiting for the command queue, frames have no surface until then");
                }
                false
            }
            DeviceStart::Failed => {
                self.device_failed = true;
                false
            }
        }
    }

    fn warn_target(warned: &mut bool, error: &windows::core::Error) {
        if !*warned {
            *warned = true;
            curia::warn!("overlay could not make the targets of the game's swap chain", {
                result: format!("{:#x}", error.code().0)
            });
        }
    }

    fn bind(
        &mut self,
        state: &LoaderState,
        chain: &IDXGISwapChain,
        desc: &DXGI_SWAP_CHAIN_DESC,
        window: usize,
        device: usize,
    ) {
        self.targets.bind(chain, desc);
        self.bound_window = window;
        self.bound_device = device;
        self.device_failed = false;
        self.target_warned = false;
        self.chain_d3d12 = Self::d3d12_of(chain);
        self.queue_logged = false;
        let (width, height) = self.targets.size();
        state.window().set_buffer(AmmlSize { width, height });
        state.window().attach(window);
        curia::info!("overlay bound", {
            hwnd: window,
            title: Self::title(window),
            width: width,
            height: height,
            format: self.targets.format(),
            buffers: desc.BufferCount,
            d3d12: self.chain_d3d12 != 0
        });
    }

    // Where the surface of the bound chain ends: the device and the kept queue go. A new chain, also on the same
    // device, may present on a new queue; drawing through the old one removed the device in a test.
    fn lose_surface(&mut self, queue: &QueueSlot, reason: AmmlSurfaceLoss) {
        curia::info!("overlay surface lost", { reason: reason.0, device_epoch: self.epoch });
        self.drop_device();
        queue.clear();
        self.chain_d3d12 = 0;
        self.queue_logged = false;
    }

    // The staging textures go, then the context is cleared and flushed, then the device goes.
    fn drop_device(&mut self) {
        let dropped = self.capture.release();
        self.dropped.extend(dropped);
        if let Some(device) = self.device.take() {
            device.clear();
        }
        self.epoch = 0;
        self.device_failed = false;
        self.queue_wait_logged = false;
        self.target_warned = false;
    }

    fn unbind(&mut self) {
        self.targets.forget();
        self.bound_window = 0;
        self.bound_device = 0;
        self.chain_d3d12 = 0;
        self.queue_logged = false;
    }

    // Once per bind of a D3D12 chain: whether the queue kept from `ExecuteCommandLists` belongs to the chain's
    // device. DXGI does not give the chain's own queue, so this is the closest check the loader can make.
    fn note_queue(&mut self, queue: &QueueSlot) {
        if self.queue_logged || self.chain_d3d12 == 0 {
            return;
        }
        let Some(kept) = queue.get() else {
            return;
        };
        self.queue_logged = true;
        let mut kept_device: Option<ID3D12Device> = None;
        // SAFETY: the kept queue is live and the out pointer is a writable local.
        let kept_device = unsafe { kept.GetDevice(&mut kept_device) }.ok().and(kept_device);
        let kept_device = kept_device.map_or(0, |device| device.as_raw().expose_provenance());
        curia::info!("overlay queue", {
            kept: format!("{:#x}", kept.as_raw().expose_provenance()),
            chain_device: format!("{:#x}", self.chain_d3d12),
            same_device: kept_device == self.chain_d3d12
        });
    }

    // Identity only: the reference taken here is released on return; 0 when the chain gives no device.
    fn device_of(chain: &IDXGISwapChain) -> usize {
        // SAFETY: the chain is live for the call that called this.
        unsafe { chain.GetDevice::<IUnknown>() }.map_or(0, |device| device.as_raw().expose_provenance())
    }

    // Identity only: the D3D12 device of a D3D12 chain; 0 for a D3D11 chain.
    fn d3d12_of(chain: &IDXGISwapChain) -> usize {
        // SAFETY: as for `device_of`.
        unsafe { chain.GetDevice::<ID3D12Device>() }.map_or(0, |device| device.as_raw().expose_provenance())
    }

    // True when the device of `chain` reports itself removed: `GetDeviceRemovedReason` is not `S_OK`. A chain with
    // neither a D3D11 nor a D3D12 device counts as not removed; it fails later, when its device is made.
    fn is_removed(chain: &IDXGISwapChain) -> bool {
        // SAFETY: the chain is live for the call that called this.
        if let Ok(device) = unsafe { chain.GetDevice::<ID3D11Device>() } {
            // SAFETY: the device is live.
            return unsafe { device.GetDeviceRemovedReason() }.is_err();
        }
        // SAFETY: as above.
        if let Ok(device) = unsafe { chain.GetDevice::<ID3D12Device>() } {
            // SAFETY: the device is live.
            return unsafe { device.GetDeviceRemovedReason() }.is_err();
        }
        false
    }

    fn is_window(hwnd: usize) -> bool {
        // SAFETY: a handle that is not a window gives false.
        unsafe { IsWindow(Some(WindowService::handle(hwnd))) }.as_bool()
    }

    fn title(hwnd: usize) -> String {
        let mut text = [0u16; Self::TITLE_UNITS];
        let window: HWND = WindowService::handle(hwnd);
        // SAFETY: `text` is a writable local; a handle that is not a window gives 0. The call sends no message, so the
        // render thread never waits for the window thread.
        let length = unsafe { InternalGetWindowText(window, &mut text) };
        let length = usize::try_from(length).unwrap_or(0).min(text.len());
        String::from_utf16_lossy(&text[..length])
    }
}
