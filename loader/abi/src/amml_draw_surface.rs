use core::ffi::c_void;

/// What a `FramePresent` listener draws on: the D3D11 objects of the presented back buffer.
///
/// `device` is an `ID3D11Device`, `context` an `ID3D11DeviceContext`, `target` an `ID3D11RenderTargetView` of
/// the back buffer and `back_buffer` its `ID3D11Resource`. `wrapped` is 1 when they wrap a D3D12 back buffer
/// (D3D11On12). `format` is the `DXGI_FORMAT` of the back buffer. All pointers are null when there is no
/// surface. They are valid during the event only and are not to be kept or given an extra reference; objects a
/// mod makes on `device` live until `SurfaceLost`, and `device_epoch` changes with each new device.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AmmlDrawSurface {
    pub size: u32,
    pub version: u32,
    pub width: u32,
    pub height: u32,
    pub format: u32,
    pub wrapped: u8,
    pub _reserved: [u8; 3],
    pub device_epoch: u64,
    pub device: *mut c_void,
    pub context: *mut c_void,
    pub target: *mut c_void,
    pub back_buffer: *mut c_void,
}

impl AmmlDrawSurface {
    /// `size` of a v1 draw surface. A literal, so it keeps its meaning when a later version adds fields.
    pub const SIZE_V1: u32 = 64;
}

const _: () = assert!(core::mem::size_of::<AmmlDrawSurface>() == AmmlDrawSurface::SIZE_V1 as usize);
