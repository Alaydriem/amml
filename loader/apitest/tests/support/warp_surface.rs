// The WARP device and its texture are D3D11 calls.
#![allow(unsafe_code)]

use amml_api::amml_abi::AmmlDrawSurface;
use windows::Win32::Foundation::HMODULE;
use windows::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE_WARP;
use windows::Win32::Graphics::Direct3D11::{
    D3D11_BIND_RENDER_TARGET, D3D11_CPU_ACCESS_READ, D3D11_CREATE_DEVICE_FLAG, D3D11_MAP_READ,
    D3D11_MAPPED_SUBRESOURCE, D3D11_SDK_VERSION, D3D11_TEXTURE2D_DESC, D3D11_USAGE_DEFAULT, D3D11_USAGE_STAGING,
    D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext, ID3D11RenderTargetView, ID3D11Texture2D,
};
use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_SAMPLE_DESC};
use windows::core::Interface;

/// A WARP D3D11 device with one render target texture, standing in for the game's back buffer in the
/// `FramePresent` events a test raises.
pub struct WarpSurface {
    pub device: ID3D11Device,
    pub context: ID3D11DeviceContext,
    pub texture: ID3D11Texture2D,
    pub target: ID3D11RenderTargetView,
}

impl WarpSurface {
    pub const SIDE: u32 = 256;

    pub fn new() -> Self {
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
        let mut texture = None;
        // SAFETY: the description and the out pointer are live locals.
        unsafe { device.CreateTexture2D(&Self::desc(D3D11_BIND_RENDER_TARGET.0 as u32), None, Some(&mut texture)) }
            .unwrap();
        let texture = texture.unwrap();
        let mut target = None;
        // SAFETY: `texture` is a render target texture of `device`; the out pointer is a live local.
        unsafe { device.CreateRenderTargetView(&texture, None, Some(&mut target)) }.unwrap();
        Self { device, context, texture, target: target.unwrap() }
    }

    fn desc(bind: u32) -> D3D11_TEXTURE2D_DESC {
        D3D11_TEXTURE2D_DESC {
            Width: Self::SIDE,
            Height: Self::SIDE,
            MipLevels: 1,
            ArraySize: 1,
            Format: DXGI_FORMAT_R8G8B8A8_UNORM,
            SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: bind,
            CPUAccessFlags: 0,
            MiscFlags: 0,
        }
    }

    pub fn surface(&self) -> AmmlDrawSurface {
        AmmlDrawSurface {
            size: AmmlDrawSurface::SIZE_V1,
            version: 1,
            width: Self::SIDE,
            height: Self::SIDE,
            format: DXGI_FORMAT_R8G8B8A8_UNORM.0 as u32,
            wrapped: 0,
            _reserved: [0; 3],
            device_epoch: 1,
            device: self.device.as_raw(),
            context: self.context.as_raw(),
            target: self.target.as_raw(),
            back_buffer: self.texture.as_raw(),
        }
    }

    /// The RGBA pixel at (`x`, `y`) of the texture.
    pub fn pixel(&self, x: u32, y: u32) -> [u8; 4] {
        let desc = D3D11_TEXTURE2D_DESC {
            Usage: D3D11_USAGE_STAGING,
            CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32,
            ..Self::desc(0)
        };
        let mut staging = None;
        // SAFETY: the description and the out pointer are live locals.
        unsafe { self.device.CreateTexture2D(&desc, None, Some(&mut staging)) }.unwrap();
        let staging = staging.unwrap();
        let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
        // SAFETY: both textures belong to the device, have the same size and format, and the staging one is
        // mapped for reading only while the bytes are copied out.
        unsafe {
            self.context.CopyResource(&staging, &self.texture);
            self.context.Map(&staging, 0, D3D11_MAP_READ, 0, Some(&mut mapped)).unwrap();
            let at = (y * mapped.RowPitch + x * 4) as usize;
            let bytes = std::slice::from_raw_parts(mapped.pData.cast::<u8>().add(at), 4);
            let pixel = [bytes[0], bytes[1], bytes[2], bytes[3]];
            self.context.Unmap(&staging, 0);
            pixel
        }
    }
}
