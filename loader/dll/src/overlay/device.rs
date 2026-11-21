use windows::Win32::Graphics::Direct3D11::{D3D11_CREATE_DEVICE_BGRA_SUPPORT, ID3D11Device, ID3D11DeviceContext};
use windows::Win32::Graphics::Direct3D11on12::{D3D11On12CreateDevice, ID3D11On12Device};
use windows::Win32::Graphics::Direct3D12::ID3D12Device;
use windows::Win32::Graphics::Dxgi::IDXGISwapChain;
use windows::core::{IUnknown, Interface};

use super::{DeviceStart, QueueSlot};

/// The D3D11 device listeners draw with. On a D3D11 chain it is the game's own device and immediate context; on a
/// D3D12 chain it is a D3D11On12 device made on the chain's D3D12 device, which submits through the kept DIRECT
/// queue. Render thread only; it lives until the surface of its epoch is lost.
pub struct OverlayDevice {
    pub device: ID3D11Device,
    pub context: ID3D11DeviceContext,
    /// Set on the D3D12 path.
    pub on12: Option<ID3D11On12Device>,
}

impl OverlayDevice {
    /// The device for `chain`: its own D3D11 device, or a D3D11On12 device on the queue `queue` holds, which must
    /// belong to the chain's device.
    pub fn start(chain: &IDXGISwapChain, queue: &QueueSlot) -> DeviceStart {
        // SAFETY: the chain is live for the Present that called this.
        if let Ok(device) = unsafe { chain.GetDevice::<ID3D11Device>() } {
            // SAFETY: the device is live.
            return match unsafe { device.GetImmediateContext() } {
                Ok(context) => DeviceStart::Ready(Self { device, context, on12: None }),
                Err(_) => {
                    curia::error!("overlay could not get the game's D3D11 context");
                    DeviceStart::Failed
                }
            };
        }
        Self::start_on12(chain, queue)
    }

    pub fn is_on12(&self) -> bool {
        self.on12.is_some()
    }

    /// The path name for the log.
    pub fn path(&self) -> &'static str {
        if self.is_on12() { "D3D12 through D3D11On12" } else { "D3D11" }
    }

    /// Unbinds everything from the context and sends its work, so no object stays bound or queued. On the D3D12
    /// path only: the D3D11 context is the game's, and the game's state is not the loader's to clear.
    pub fn clear(&self) {
        if self.is_on12() {
            // SAFETY: the D3D11On12 context is the loader's own and is used on the render thread only.
            unsafe {
                self.context.ClearState();
                self.context.Flush();
            }
        }
    }

    fn start_on12(chain: &IDXGISwapChain, slot: &QueueSlot) -> DeviceStart {
        let Some(queue) = slot.get() else {
            return DeviceStart::Waiting;
        };
        // SAFETY: the chain is live for the Present that called this.
        let Ok(chain_device) = (unsafe { chain.GetDevice::<ID3D12Device>() }) else {
            curia::error!("overlay swap chain has neither a D3D11 nor a D3D12 device");
            return DeviceStart::Failed;
        };
        let mut queue_device: Option<ID3D12Device> = None;
        // SAFETY: the queue is live and the out pointer is a writable local.
        let queue_device = unsafe { queue.GetDevice(&mut queue_device) }.ok().and(queue_device);
        if queue_device.as_ref() != Some(&chain_device) {
            curia::info!("overlay drops a kept queue of another device");
            slot.clear();
            return DeviceStart::Waiting;
        }
        let Ok(queue) = queue.cast::<IUnknown>() else {
            return DeviceStart::Failed;
        };
        let queues = [Some(queue)];
        let mut device: Option<ID3D11Device> = None;
        let mut context: Option<ID3D11DeviceContext> = None;
        // SAFETY: the device and queue are live, and the out pointers are writable locals.
        let made = unsafe {
            D3D11On12CreateDevice(
                &chain_device,
                D3D11_CREATE_DEVICE_BGRA_SUPPORT.0,
                None,
                Some(&queues),
                0,
                Some(&mut device),
                Some(&mut context),
                None,
            )
        };
        let on12 = device.as_ref().and_then(|device| device.cast::<ID3D11On12Device>().ok());
        match (made, device, context, on12) {
            (Ok(()), Some(device), Some(context), Some(on12)) => {
                DeviceStart::Ready(Self { device, context, on12: Some(on12) })
            }
            (made, ..) => {
                curia::error!("overlay could not create a D3D11On12 device", {
                    result: format!("{:#x}", made.err().map_or(0, |error| error.code().0))
                });
                DeviceStart::Failed
            }
        }
    }
}
