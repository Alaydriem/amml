use std::cell::Cell;
use std::marker::PhantomData;

use amml_abi::{AmmlDrawSurface, OverlayCopyBackBufferFn, OverlayPrepareSurfaceFn};
#[cfg(feature = "d3d")]
use windows::Win32::Graphics::Direct3D11::{
    ID3D11Device, ID3D11DeviceContext, ID3D11RenderTargetView, ID3D11Texture2D,
};

#[cfg(feature = "d3d")]
use crate::ApiError;
use crate::{ComRef, Size};

#[cfg(feature = "d3d")]
type Device = ID3D11Device;
#[cfg(feature = "d3d")]
type Context = ID3D11DeviceContext;
#[cfg(feature = "d3d")]
type Target = ID3D11RenderTargetView;
#[cfg(not(feature = "d3d"))]
type Device = std::ffi::c_void;
#[cfg(not(feature = "d3d"))]
type Context = std::ffi::c_void;
#[cfg(not(feature = "d3d"))]
type Target = std::ffi::c_void;

/// What a `FramePresent` listener draws on, from `FramePresentExt::surface`. It lives no longer than
/// the listener call (`'f`), on the render thread.
///
/// `device`, `context` and `target` are frame handles: valid during the call only. Objects a mod makes
/// on `device` live until `SurfaceLost`; `device_epoch` changes with each new device. On a surface with
/// `wrapped() == false` the context is the game's own immediate context.
///
/// On a loader that serves `FramePresent::ON_DEMAND_VERSION` the loader makes the frame's surface only
/// when a listener needs it: the first `context()`, `target()` or `copy_back_buffer` of the frame asks
/// for it, and a frame where no listener calls them costs no surface. A listener that has nothing to
/// draw calls none of them. When the loader cannot make the surface, `context()` and `target()` give a
/// `ComRef` whose `with` gives `None`.
///
/// Without the `d3d` feature the handles are `ComRef<'f, c_void>` and give only `as_raw`; with it they
/// are `ComRef`s of the `windows` D3D11 types (`ID3D11Device`, `ID3D11DeviceContext`,
/// `ID3D11RenderTargetView`).
pub struct DrawSurface<'f> {
    // The surface as the loader lent it, and the full one once `prepare_surface` made it.
    surface: Cell<AmmlDrawSurface>,
    device: ComRef<'f, Device>,
    #[cfg_attr(not(feature = "d3d"), allow(dead_code))]
    copy_back_buffer: Option<OverlayCopyBackBufferFn>,
    prepare_surface: Option<OverlayPrepareSurfaceFn>,
    // The loader could not make the surface of this frame; it is not asked again.
    refused: Cell<bool>,
    _frame: PhantomData<&'f ()>,
}

impl<'f> DrawSurface<'f> {
    /// The surface of `surface`; `None` when it is shorter than v1 or has no device, and when any of
    /// its pointers is null without `prepare_surface` (a loader that lends only full surfaces). With
    /// `prepare_surface`, a surface with a device and a null back buffer is made on first use.
    ///
    /// # Safety
    /// The pointers of `surface` are null or live D3D11 objects of the documented types for `'f`, on
    /// this thread; `prepare_surface`, when given, is the loader's function for the running
    /// `FramePresent` call, which keeps the surface it makes live for `'f`.
    #[allow(unsafe_code)]
    pub(crate) unsafe fn new(
        surface: AmmlDrawSurface,
        copy_back_buffer: Option<OverlayCopyBackBufferFn>,
        prepare_surface: Option<OverlayPrepareSurfaceFn>,
    ) -> Option<Self> {
        if surface.size < AmmlDrawSurface::SIZE_V1 {
            return None;
        }
        // SAFETY: the caller's contract, for the device pointer and its type.
        let device = unsafe { ComRef::from_loader(surface.device) }?;
        let on_demand = surface.back_buffer.is_null() && prepare_surface.is_some();
        if !on_demand && !Self::is_full(&surface) {
            return None;
        }
        Some(Self {
            surface: Cell::new(surface),
            device,
            copy_back_buffer,
            prepare_surface,
            refused: Cell::new(false),
            _frame: PhantomData,
        })
    }

    /// The back buffer size.
    pub fn size(&self) -> Size {
        let surface = self.surface.get();
        Size { width: surface.width, height: surface.height }
    }

    /// The `DXGI_FORMAT` of the back buffer.
    pub fn format(&self) -> u32 {
        self.surface.get().format
    }

    /// Whether the objects wrap a D3D12 back buffer (D3D11On12); the context is then the loader's.
    pub fn wrapped(&self) -> bool {
        self.surface.get().wrapped != 0
    }

    /// The epoch of the device; it changes with each new device, after `SurfaceLost`.
    pub fn device_epoch(&self) -> u64 {
        self.surface.get().device_epoch
    }

    /// The `ID3D11Device`. Objects made on it live until `SurfaceLost`.
    pub fn device(&self) -> ComRef<'f, Device> {
        self.device
    }

    /// Whether the frame has a full surface to draw on: asks the loader for it when it was not made yet
    /// (as `context()` and `target()` do). False when the loader could not make it; `context()` and
    /// `target()` then give null pointers.
    pub fn ready(&self) -> bool {
        self.full().is_some()
    }

    /// The `ID3D11DeviceContext` to draw with, on the render thread inside the call only. Asks the
    /// loader for the frame's surface when it was not made yet.
    pub fn context(&self) -> ComRef<'f, Context> {
        match self.full() {
            // SAFETY: a full surface holds a live context for `'f` (the contract of `new`).
            #[allow(unsafe_code)]
            Some(surface) => unsafe { ComRef::from_loader(surface.context) }.unwrap_or_else(ComRef::absent),
            None => ComRef::absent(),
        }
    }

    /// The `ID3D11RenderTargetView` of the back buffer. Asks the loader for the frame's surface when it
    /// was not made yet.
    pub fn target(&self) -> ComRef<'f, Target> {
        match self.full() {
            // SAFETY: a full surface holds a live target for `'f` (the contract of `new`).
            #[allow(unsafe_code)]
            Some(surface) => unsafe { ComRef::from_loader(surface.target) }.unwrap_or_else(ComRef::absent),
            None => ComRef::absent(),
        }
    }

    // The full surface: the lent one, or the one the loader makes now on the first ask of the frame.
    fn full(&self) -> Option<AmmlDrawSurface> {
        let surface = self.surface.get();
        if Self::is_full(&surface) {
            return Some(surface);
        }
        if self.refused.get() {
            return None;
        }
        let prepare = self.prepare_surface?;
        let mut out = std::mem::MaybeUninit::<AmmlDrawSurface>::zeroed();
        // SAFETY: `out` is writable for the call; the loader checks that this runs inside its
        // `FramePresent` raise on this thread.
        #[allow(unsafe_code)]
        let status = unsafe { prepare(out.as_mut_ptr()) };
        // SAFETY: all zero bytes are a valid surface (null pointers), and the loader writes a whole
        // one on `OK`.
        #[allow(unsafe_code)]
        let made = unsafe { out.assume_init() };
        if status.is_ok() && made.size >= AmmlDrawSurface::SIZE_V1 && Self::is_full(&made) {
            self.surface.set(made);
            Some(made)
        } else {
            self.refused.set(true);
            None
        }
    }

    fn is_full(surface: &AmmlDrawSurface) -> bool {
        !surface.device.is_null()
            && !surface.context.is_null()
            && !surface.target.is_null()
            && !surface.back_buffer.is_null()
    }
}

#[cfg(feature = "d3d")]
impl DrawSurface<'_> {
    /// Copies the back buffer, as drawn so far, into `dest`: a texture made on `device` with the back
    /// buffer's size and sample count, the same format (or its sRGB / UNORM twin), one mip and one
    /// slice, `DEFAULT` usage and no CPU access. `Loader(INVALID_ARGUMENT)` for a pointer that is not
    /// a texture; `Refused` for a texture of another device, size or format, or outside the
    /// `FramePresent` call; `NotAvailable` without the overlay.
    pub fn copy_back_buffer(&self, dest: ComRef<'_, ID3D11Texture2D>) -> Result<(), ApiError> {
        let copy = self.copy_back_buffer.ok_or(ApiError::NotAvailable)?;
        // SAFETY: `dest` borrows a live texture (or is null) for the call; the loader checks it before it
        // copies.
        #[allow(unsafe_code)]
        let status = unsafe { copy(dest.as_raw()) };
        ApiError::check(status)
    }
}
