use amml_abi::AmmlDrawSurface;
use windows::Win32::Graphics::Direct3D11::{ID3D11Device, ID3D11DeviceContext, ID3D11RenderTargetView, ID3D11Resource};
use windows::Win32::Graphics::Direct3D11on12::ID3D11On12Device;
use windows::core::Interface;

use super::WrappedAcquire;

/// The D3D11 objects of one frame's back buffer, each with a reference of its own for the frame: the loader's
/// device and context, the render target view and the back buffer (on D3D12, its D3D11On12 wrapper). Dropped before
/// the original Present, so the loader keeps no D3D11 back buffer between frames.
#[derive(Clone)]
pub struct DrawSurface {
    pub device: ID3D11Device,
    pub context: ID3D11DeviceContext,
    pub target: ID3D11RenderTargetView,
    pub back_buffer: ID3D11Resource,
    /// Set on the D3D12 path: the device that made `back_buffer` as a wrapper.
    pub on12: Option<ID3D11On12Device>,
    pub width: u32,
    pub height: u32,
    pub format: u32,
    /// The sample count of the back buffer.
    pub samples: u32,
    pub epoch: u64,
}

impl DrawSurface {
    /// The surface of a present with nothing to draw on: every value 0 and every pointer null, apart from `size` and
    /// `version`.
    pub fn empty() -> AmmlDrawSurface {
        AmmlDrawSurface {
            size: AmmlDrawSurface::SIZE_V1,
            version: 1,
            width: 0,
            height: 0,
            format: 0,
            wrapped: 0,
            _reserved: [0; 3],
            device_epoch: 0,
            device: std::ptr::null_mut(),
            context: std::ptr::null_mut(),
            target: std::ptr::null_mut(),
            back_buffer: std::ptr::null_mut(),
        }
    }

    /// The surface of a present whose listeners take it on demand, before one asks: `device` (wrapped when `wrapped`),
    /// the buffer's size and format and the device epoch, with a null context, target and back buffer.
    pub fn waiting(
        device: &ID3D11Device,
        wrapped: bool,
        width: u32,
        height: u32,
        format: u32,
        epoch: u64,
    ) -> AmmlDrawSurface {
        AmmlDrawSurface {
            width,
            height,
            format,
            wrapped: u8::from(wrapped),
            device_epoch: epoch,
            device: device.as_raw(),
            ..Self::empty()
        }
    }

    /// The surface as `FramePresent` carries it. The pointers are borrowed from this value, which outlives the
    /// listeners.
    pub fn abi(&self) -> AmmlDrawSurface {
        AmmlDrawSurface {
            size: AmmlDrawSurface::SIZE_V1,
            version: 1,
            width: self.width,
            height: self.height,
            format: self.format,
            wrapped: u8::from(self.on12.is_some()),
            _reserved: [0; 3],
            device_epoch: self.epoch,
            device: self.device.as_raw(),
            context: self.context.as_raw(),
            target: self.target.as_raw(),
            back_buffer: self.back_buffer.as_raw(),
        }
    }

    /// On the D3D12 path, acquires the wrapped back buffer for the listeners; released when the value drops.
    pub fn acquire(&self) -> Option<WrappedAcquire> {
        self.on12.as_ref().map(|on12| WrappedAcquire::new(on12, &self.context, &self.back_buffer))
    }
}
