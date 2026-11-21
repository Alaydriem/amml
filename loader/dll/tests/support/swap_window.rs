use windows::Win32::Foundation::{HMODULE, HWND};
use windows::Win32::Graphics::Direct3D::{D3D_DRIVER_TYPE_WARP, D3D_FEATURE_LEVEL_11_0};
use windows::Win32::Graphics::Direct3D11::{
    D3D11_CREATE_DEVICE_FLAG, D3D11_SDK_VERSION, D3D11CreateDeviceAndSwapChain, ID3D11Texture2D,
};
use windows::Win32::Graphics::Dxgi::Common::{
    DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_FORMAT_UNKNOWN, DXGI_MODE_DESC, DXGI_SAMPLE_DESC,
};
use windows::Win32::Graphics::Dxgi::{
    DXGI_PRESENT, DXGI_PRESENT_PARAMETERS, DXGI_SWAP_CHAIN_DESC, DXGI_SWAP_CHAIN_FLAG, DXGI_SWAP_EFFECT_FLIP_DISCARD,
    DXGI_USAGE_RENDER_TARGET_OUTPUT, IDXGIAdapter, IDXGISwapChain, IDXGISwapChain1,
};
use windows::core::{HRESULT, Interface};

use super::TestWindow;

/// A hidden window with a flip swap chain on a WARP D3D11 device of two `SIZE` x `SIZE` R8G8B8A8 buffers, as the
/// game makes. The chain drops before the window.
pub struct SwapWindow {
    pub chain: IDXGISwapChain,
    pub window: TestWindow,
}

impl SwapWindow {
    pub const SIZE: u32 = 64;

    /// A chain on a window of `SIZE` x `SIZE`.
    pub fn new() -> Self {
        Self::on_window(Self::SIZE, Self::SIZE)
    }

    /// A chain of `SIZE` x `SIZE` buffers on a window whose client area is `width` x `height`.
    pub fn on_window(width: u32, height: u32) -> Self {
        let window = TestWindow::new(width, height);
        let chain = Self::chain_on(window.hwnd);
        Self { chain, window }
    }

    /// Drops the chain, then makes a new device and chain on the same window, as a game that rebuilds its device.
    /// The old chain must be gone first: a window takes one flip chain at a time.
    pub fn replace_chain(self) -> Self {
        let Self { chain, window } = self;
        drop(chain);
        let chain = Self::chain_on(window.hwnd);
        Self { chain, window }
    }

    fn chain_on(hwnd: HWND) -> IDXGISwapChain {
        let desc = DXGI_SWAP_CHAIN_DESC {
            BufferDesc: DXGI_MODE_DESC {
                Width: Self::SIZE,
                Height: Self::SIZE,
                Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                ..Default::default()
            },
            SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
            BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
            BufferCount: 2,
            OutputWindow: hwnd,
            Windowed: true.into(),
            SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
            Flags: 0,
        };
        let mut chain = None;
        // SAFETY: `desc` and the out pointers live for the call.
        unsafe {
            D3D11CreateDeviceAndSwapChain(
                None::<&IDXGIAdapter>,
                D3D_DRIVER_TYPE_WARP,
                HMODULE::default(),
                D3D11_CREATE_DEVICE_FLAG(0),
                Some(&[D3D_FEATURE_LEVEL_11_0]),
                D3D11_SDK_VERSION,
                Some(&desc),
                Some(&mut chain),
                None,
                None,
                None,
            )
        }
        .unwrap();
        chain.unwrap()
    }

    /// Presents; the result is not checked, since a hidden window may report itself occluded.
    pub fn present(&self) {
        let _ = self.present_with(DXGI_PRESENT(0));
    }

    /// Presents with `flags` and gives the result.
    pub fn present_with(&self, flags: DXGI_PRESENT) -> HRESULT {
        // SAFETY: the chain is live.
        unsafe { self.chain.Present(0, flags) }
    }

    pub fn present1(&self) {
        let chain: IDXGISwapChain1 = self.chain.cast().unwrap();
        let params = DXGI_PRESENT_PARAMETERS::default();
        // SAFETY: the chain and `params` are live.
        let _ = unsafe { chain.Present1(0, DXGI_PRESENT(0), &params) };
    }

    /// `ResizeBuffers` with the buffer count and format kept.
    pub fn resize(&self, width: u32, height: u32) -> windows::core::Result<()> {
        // SAFETY: the chain is live; 0 buffers and an unknown format keep what the chain has.
        unsafe { self.chain.ResizeBuffers(0, width, height, DXGI_FORMAT_UNKNOWN, DXGI_SWAP_CHAIN_FLAG(0)) }
    }

    /// The buffer size the chain has now.
    pub fn buffer_size(&self) -> (u32, u32) {
        // SAFETY: the chain is live.
        let desc = unsafe { self.chain.GetDesc() }.unwrap();
        (desc.BufferDesc.Width, desc.BufferDesc.Height)
    }

    /// A reference to back buffer 0; while it lives, `ResizeBuffers` fails.
    pub fn hold_buffer(&self) -> ID3D11Texture2D {
        // SAFETY: the chain is live and has buffer 0.
        unsafe { self.chain.GetBuffer(0) }.unwrap()
    }

    /// How many times Present or Present1 of this chain reached DXGI.
    pub fn dxgi_present_count(&self) -> u32 {
        // SAFETY: the chain is live.
        unsafe { self.chain.GetLastPresentCount() }.unwrap()
    }

    /// The handle as the loader takes it.
    pub fn address(&self) -> usize {
        self.window.address()
    }
}
