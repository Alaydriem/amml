use windows::Win32::Graphics::Direct3D11::{ID3D11DeviceContext, ID3D11Resource};
use windows::Win32::Graphics::Direct3D11on12::ID3D11On12Device;

/// A back-buffer wrapper acquired for the listeners of one Present. On drop, also when a panic unwinds through the
/// draw, the targets are unbound, the wrapper is released and the context flushed, so the D3D12 back buffer is back
/// in the `PRESENT` state before the original Present and nothing holds it bound.
pub struct WrappedAcquire {
    on12: ID3D11On12Device,
    context: ID3D11DeviceContext,
    resources: [Option<ID3D11Resource>; 1],
}

impl WrappedAcquire {
    pub fn new(on12: &ID3D11On12Device, context: &ID3D11DeviceContext, wrapped: &ID3D11Resource) -> Self {
        let resources = [Some(wrapped.clone())];
        // SAFETY: the wrapper is live and made by `on12`; it is released when this value drops, in the same Present.
        unsafe { on12.AcquireWrappedResources(&resources) };
        Self { on12: on12.clone(), context: context.clone(), resources }
    }
}

impl Drop for WrappedAcquire {
    fn drop(&mut self) {
        // SAFETY: the wrapper was acquired in `new` on this thread; binding no target takes no object.
        unsafe {
            self.context.OMSetRenderTargets(None, None);
            self.on12.ReleaseWrappedResources(&self.resources);
            self.context.Flush();
        }
    }
}
