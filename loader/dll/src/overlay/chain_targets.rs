use windows::Win32::Foundation::E_POINTER;
use windows::Win32::Graphics::Direct3D11::{
    D3D11_BIND_RENDER_TARGET, D3D11_TEXTURE2D_DESC, ID3D11Device, ID3D11RenderTargetView, ID3D11Resource,
    ID3D11Texture2D,
};
use windows::Win32::Graphics::Direct3D11on12::{D3D11_RESOURCE_FLAGS, ID3D11On12Device};
use windows::Win32::Graphics::Direct3D12::{D3D12_RESOURCE_STATE_PRESENT, ID3D12Resource};
use windows::Win32::Graphics::Dxgi::{DXGI_SWAP_CHAIN_DESC, IDXGISwapChain, IDXGISwapChain3};
use windows::core::{GUID, Interface};

/// The bound swap chain as the overlay holds it: its identity (no reference to the chain is held), its buffer size,
/// format and sample count. The identity is the address and a mark: at the bind the chain gets the bind number as
/// private data (plain bytes, no reference), so a new chain made at the freed address of the bound one is not taken
/// for it. No back buffer is held between frames on either path: `native` (D3D11) and `wrapped`
/// (D3D12) make the objects of one frame, which drop before the original Present.
///
/// A held back buffer keeps a flip chain alive, and a window takes one flip chain at a time: a game that rebuilds its
/// chain on the window would get `E_ACCESSDENIED`. Holding nothing between frames also means no back buffer is held
/// across `ResizeBuffers`.
#[derive(Debug, Default)]
pub struct ChainTargets {
    // Identity only: the bound chain, with no reference held; 0 when none is bound.
    chain: usize,
    // The mark the bound chain carries; it counts the binds of the process.
    mark: u64,
    // DXGI took the mark at the bind; when it refused, the address alone tells the bound chain.
    marked: bool,
    width: u32,
    height: u32,
    format: u32,
    samples: u32,
}

impl ChainTargets {
    // The private data name of the bind mark.
    const MARK: GUID = GUID::from_u128(0x5c1f_7a2e_9b44_4d0e_a6f3_2e8d_41c0_7b19);

    pub const fn new() -> Self {
        Self { chain: 0, mark: 0, marked: false, width: 0, height: 0, format: 0, samples: 0 }
    }

    /// Binds `chain`, described by `desc`, and marks it with the next bind number.
    pub fn bind(&mut self, chain: &IDXGISwapChain, desc: &DXGI_SWAP_CHAIN_DESC) {
        let mark = self.mark + 1;
        // SAFETY: the chain is live for the Present that called this; the data is a local of the given size, which
        // DXGI copies.
        let marked = unsafe { chain.SetPrivateData(&Self::MARK, 8, (&raw const mark).cast()) };
        if marked.is_err() {
            curia::warn!("overlay could not mark the bound swap chain, a chain at its address passes as it");
        }
        self.chain = chain.as_raw().expose_provenance();
        self.mark = mark;
        self.marked = marked.is_ok();
        self.set_desc(desc);
    }

    /// Forgets the bound chain. The bind number goes on.
    pub fn forget(&mut self) {
        *self = Self { mark: self.mark, ..Self::new() };
    }

    /// The size, format and sample count the bound chain has after a `ResizeBuffers` or a bind.
    pub fn set_desc(&mut self, desc: &DXGI_SWAP_CHAIN_DESC) {
        self.width = desc.BufferDesc.Width;
        self.height = desc.BufferDesc.Height;
        self.format = desc.BufferDesc.Format.0 as u32;
        self.samples = desc.SampleDesc.Count;
    }

    /// The size and format a frame read from the back buffer itself, when they differ from the kept ones (a new chain
    /// at the freed address of the bound one). True when they changed.
    pub fn note_buffer(&mut self, width: u32, height: u32, format: u32, samples: u32) -> bool {
        let changed = (self.width, self.height, self.format, self.samples) != (width, height, format, samples);
        (self.width, self.height, self.format, self.samples) = (width, height, format, samples);
        changed
    }

    pub fn is_bound(&self) -> bool {
        self.chain != 0
    }

    /// True when `chain` is the bound chain: the same address and the mark of the bind (the address alone when the
    /// chain could not be marked).
    pub fn is_chain(&self, chain: &IDXGISwapChain) -> bool {
        if self.chain == 0 || self.chain != chain.as_raw().expose_provenance() {
            return false;
        }
        if !self.marked {
            return true;
        }
        let mut mark = 0u64;
        let mut size = 8u32;
        // SAFETY: the chain is live for the call that called this; `mark` and `size` are writable locals and `size`
        // is the room in `mark`.
        let read = unsafe { chain.GetPrivateData(&Self::MARK, &mut size, (&raw mut mark).cast()) };
        read.is_ok() && size == 8 && mark == self.mark
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// The `DXGI_FORMAT` value of the bound chain's buffers.
    pub fn format(&self) -> u32 {
        self.format
    }

    /// The sample count of the bound chain's buffers.
    pub fn samples(&self) -> u32 {
        self.samples
    }

    /// A D3D11On12 wrapper (in the `PRESENT` state) of the back buffer `chain` draws into now, and a render target
    /// view of it on `device` (the D3D11 device of `on12`), for one frame.
    pub fn wrapped(
        device: &ID3D11Device,
        on12: &ID3D11On12Device,
        chain: &IDXGISwapChain,
    ) -> windows::core::Result<(ID3D11Resource, ID3D11RenderTargetView)> {
        let chain3 = chain.cast::<IDXGISwapChain3>()?;
        // SAFETY: the chain is live for the Present that called this.
        let index = unsafe { chain3.GetCurrentBackBufferIndex() };
        // SAFETY: the chain is live and `index` is its current back buffer.
        let buffer: ID3D12Resource = unsafe { chain.GetBuffer(index) }?;
        let flags = D3D11_RESOURCE_FLAGS { BindFlags: D3D11_BIND_RENDER_TARGET.0 as u32, ..Default::default() };
        let mut wrapped: Option<ID3D11Resource> = None;
        // SAFETY: the buffer is a live back buffer, in the PRESENT state outside a Present; the flags and the out
        // pointer live for the call.
        unsafe {
            on12.CreateWrappedResource(
                &buffer,
                &flags,
                D3D12_RESOURCE_STATE_PRESENT,
                D3D12_RESOURCE_STATE_PRESENT,
                &mut wrapped,
            )
        }?;
        let wrapped = wrapped.ok_or(E_POINTER)?;
        let view = Self::view(device, &wrapped)?;
        Ok((wrapped, view))
    }

    /// The back buffer of a D3D11 `chain` (buffer 0, which a flip chain always shows as its current back buffer),
    /// its description and a new render target view of it on `device`, for one frame.
    pub fn native(
        device: &ID3D11Device,
        chain: &IDXGISwapChain,
    ) -> windows::core::Result<(ID3D11Resource, D3D11_TEXTURE2D_DESC, ID3D11RenderTargetView)> {
        // SAFETY: the chain is live for the Present that called this and has buffer 0.
        let buffer: ID3D11Texture2D = unsafe { chain.GetBuffer(0) }?;
        let mut desc = D3D11_TEXTURE2D_DESC::default();
        // SAFETY: the texture is live and `desc` is writable for the call.
        unsafe { buffer.GetDesc(&mut desc) };
        let resource: ID3D11Resource = buffer.cast()?;
        let view = Self::view(device, &resource)?;
        Ok((resource, desc, view))
    }

    /// The description of the back buffer of a D3D11 `chain` (buffer 0); the buffer's reference is released on return.
    pub fn native_desc(chain: &IDXGISwapChain) -> windows::core::Result<D3D11_TEXTURE2D_DESC> {
        // SAFETY: the chain is live for the Present that called this and has buffer 0.
        let buffer: ID3D11Texture2D = unsafe { chain.GetBuffer(0) }?;
        let mut desc = D3D11_TEXTURE2D_DESC::default();
        // SAFETY: the texture is live and `desc` is writable for the call.
        unsafe { buffer.GetDesc(&mut desc) };
        Ok(desc)
    }

    fn view(device: &ID3D11Device, resource: &ID3D11Resource) -> windows::core::Result<ID3D11RenderTargetView> {
        let mut view: Option<ID3D11RenderTargetView> = None;
        // SAFETY: the resource is a live back buffer made with render target binding; the out pointer is a local.
        unsafe { device.CreateRenderTargetView(resource, None, Some(&mut view)) }?;
        view.ok_or_else(|| E_POINTER.into())
    }
}
