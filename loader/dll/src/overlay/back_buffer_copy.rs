use std::cell::RefCell;
use std::ffi::c_void;

use amml_abi::AmmlStatus;
use windows::Win32::Graphics::Direct3D11::{D3D11_TEXTURE2D_DESC, D3D11_USAGE_DEFAULT, ID3D11Texture2D};
use windows::Win32::Graphics::Dxgi::Common::{
    DXGI_FORMAT, DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_FORMAT_B8G8R8A8_UNORM_SRGB, DXGI_FORMAT_B8G8R8X8_UNORM,
    DXGI_FORMAT_B8G8R8X8_UNORM_SRGB, DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_FORMAT_R8G8B8A8_UNORM_SRGB,
};
use windows::core::{IUnknown, Interface};

use super::DrawSurface;

thread_local! {
    // The surface of the `FramePresent` this thread is raising now; none outside one.
    static FRAME: RefCell<Option<DrawSurface>> = const { RefCell::new(None) };
}

/// The copy of this frame's back buffer into a mod's texture (`AmmlOverlayTable::copy_back_buffer`). It works only
/// on the render thread inside a `FramePresent` raise: the value `open` gives marks that window and closes it when it
/// drops, also when a panic unwinds through the listeners.
pub struct BackBufferCopy {
    open: bool,
}

impl BackBufferCopy {
    /// Opens the copy window for `surface` on this thread until the value drops.
    pub fn open(surface: &DrawSurface) -> Self {
        let open = FRAME.try_with(|frame| *frame.borrow_mut() = Some(surface.clone())).is_ok();
        Self { open }
    }

    /// Copies this frame's back buffer into the `ID3D11Texture2D` at `dest`. `INVALID_ARGUMENT` for a null pointer or
    /// an object that is not a 2D texture. `REFUSED` outside a `FramePresent` raise on this thread, and for a texture
    /// of another device, of another size, of a format other than the back buffer's (or its sRGB or UNORM twin), of
    /// another sample count than the back buffer's, with several mips or array slices, with CPU access or other than
    /// default usage, or the back buffer itself.
    ///
    /// # Safety
    /// A non-null `dest` points to a live COM object for the call.
    pub unsafe fn copy(dest: *mut c_void) -> AmmlStatus {
        if dest.is_null() {
            return AmmlStatus::INVALID_ARGUMENT;
        }
        let Some(surface) = FRAME.try_with(|frame| frame.borrow().clone()).ok().flatten() else {
            return AmmlStatus::REFUSED;
        };
        // SAFETY: `dest` is non-null and the caller promises it is a live COM object for the call.
        let Some(dest) = (unsafe { IUnknown::from_raw_borrowed(&dest) }) else {
            return AmmlStatus::INVALID_ARGUMENT;
        };
        let Ok(texture) = dest.cast::<ID3D11Texture2D>() else {
            return AmmlStatus::INVALID_ARGUMENT;
        };
        let status = Self::check(&surface, &texture);
        if status != AmmlStatus::OK {
            return status;
        }
        // SAFETY: both resources are live and of one device, the same size, format family and sample count, one mip and
        // one slice; on the D3D12 path the back buffer is acquired for the raise this call runs in.
        unsafe { surface.context.CopyResource(&texture, &surface.back_buffer) };
        AmmlStatus::OK
    }

    fn check(surface: &DrawSurface, texture: &ID3D11Texture2D) -> AmmlStatus {
        if Self::identity(texture) == Self::identity(&surface.back_buffer) {
            return AmmlStatus::REFUSED;
        }
        // SAFETY: the texture is live.
        let same_device = unsafe { texture.GetDevice() }.is_ok_and(|device| {
            Self::identity(&device) != 0 && Self::identity(&device) == Self::identity(&surface.device)
        });
        if !same_device {
            return AmmlStatus::REFUSED;
        }
        let mut desc = D3D11_TEXTURE2D_DESC::default();
        // SAFETY: the texture is live and `desc` is writable for the call.
        unsafe { texture.GetDesc(&mut desc) };
        let fits = desc.Width == surface.width
            && desc.Height == surface.height
            && Self::same_family(desc.Format, DXGI_FORMAT(surface.format as i32))
            && desc.SampleDesc.Count == surface.samples
            && desc.MipLevels == 1
            && desc.ArraySize == 1
            && desc.Usage == D3D11_USAGE_DEFAULT
            && desc.CPUAccessFlags == 0;
        if fits { AmmlStatus::OK } else { AmmlStatus::REFUSED }
    }

    // The format itself, or its sRGB or UNORM twin: `CopyResource` copies between them, and a texture sampled as UNORM
    // reads the gamma-space bytes of an sRGB back buffer.
    fn same_family(texture: DXGI_FORMAT, back_buffer: DXGI_FORMAT) -> bool {
        texture == back_buffer || Self::unorm(texture) == Self::unorm(back_buffer)
    }

    fn unorm(format: DXGI_FORMAT) -> DXGI_FORMAT {
        match format {
            DXGI_FORMAT_R8G8B8A8_UNORM_SRGB => DXGI_FORMAT_R8G8B8A8_UNORM,
            DXGI_FORMAT_B8G8R8A8_UNORM_SRGB => DXGI_FORMAT_B8G8R8A8_UNORM,
            DXGI_FORMAT_B8G8R8X8_UNORM_SRGB => DXGI_FORMAT_B8G8R8X8_UNORM,
            other => other,
        }
    }

    // The object's `IUnknown` address, which is its COM identity; 0 when the cast fails.
    fn identity(object: &impl Interface) -> usize {
        object.cast::<IUnknown>().map_or(0, |unknown| unknown.as_raw().expose_provenance())
    }
}

impl Drop for BackBufferCopy {
    fn drop(&mut self) {
        if self.open {
            let _ = FRAME.try_with(|frame| frame.borrow_mut().take());
        }
    }
}
