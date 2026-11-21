use super::OverlayDevice;

/// What `OverlayDevice::start` made of a swap chain.
pub enum DeviceStart {
    Ready(OverlayDevice),
    /// A D3D12 chain whose queue was not seen yet, or the kept queue is of another device and was dropped; tried
    /// again at the next present.
    Waiting,
    /// No device can be made on this chain; logged, and tried again only after the next bind.
    Failed,
}
