use std::marker::PhantomData;

use amml_abi::{AmmlDrawSurface, OverlayCopyBackBufferFn};
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
/// Without the `d3d` feature the handles are `ComRef<'f, c_void>` and give only `as_raw`; with it they
/// are `ComRef`s of the `windows` D3D11 types (`ID3D11Device`, `ID3D11DeviceContext`,
/// `ID3D11RenderTargetView`).
pub struct DrawSurface<'f> {
    surface: AmmlDrawSurface,
    device: ComRef<'f, Device>,
    context: ComRef<'f, Context>,
    target: ComRef<'f, Target>,
    #[cfg_attr(not(feature = "d3d"), allow(dead_code))]
    copy_back_buffer: Option<OverlayCopyBackBufferFn>,
    _frame: PhantomData<&'f ()>,
}

impl<'f> DrawSurface<'f> {
    /// The surface of `surface`; `None` when it is shorter than v1 or any of its pointers is null.
    ///
    /// # Safety
    /// The pointers of `surface` are null or live D3D11 objects of the documented types for `'f`, on
    /// this thread.
    #[allow(unsafe_code)]
    pub(crate) unsafe fn new(
        surface: AmmlDrawSurface,
        copy_back_buffer: Option<OverlayCopyBackBufferFn>,
    ) -> Option<Self> {
        if surface.size < AmmlDrawSurface::SIZE_V1 || surface.back_buffer.is_null() {
            return None;
        }
        // SAFETY: the caller's contract, for each pointer and its type.
        let (device, context, target) = unsafe {
            (
                ComRef::from_loader(surface.device)?,
                ComRef::from_loader(surface.context)?,
                ComRef::from_loader(surface.target)?,
            )
        };
        Some(Self { surface, device, context, target, copy_back_buffer, _frame: PhantomData })
    }

    /// The back buffer size.
    pub fn size(&self) -> Size {
        Size { width: self.surface.width, height: self.surface.height }
    }

    /// The `DXGI_FORMAT` of the back buffer.
    pub fn format(&self) -> u32 {
        self.surface.format
    }

    /// Whether the objects wrap a D3D12 back buffer (D3D11On12); the context is then the loader's.
    pub fn wrapped(&self) -> bool {
        self.surface.wrapped != 0
    }

    /// The epoch of the device; it changes with each new device, after `SurfaceLost`.
    pub fn device_epoch(&self) -> u64 {
        self.surface.device_epoch
    }

    /// The `ID3D11Device`. Objects made on it live until `SurfaceLost`.
    pub fn device(&self) -> ComRef<'f, Device> {
        self.device
    }

    /// The `ID3D11DeviceContext` to draw with, on the render thread inside the call only.
    pub fn context(&self) -> ComRef<'f, Context> {
        self.context
    }

    /// The `ID3D11RenderTargetView` of the back buffer.
    pub fn target(&self) -> ComRef<'f, Target> {
        self.target
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
