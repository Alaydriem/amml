use amml_abi::AmmlSurfaceLoss;

/// What ends the draw surface at a present, found before the loader drops anything, so `SurfaceLost` comes first.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurfaceEnd {
    /// The bound window is gone; the next chain presented is bound.
    WindowGone,
    /// The bound chain reports another device.
    NewDevice,
    /// Another chain presents on the bound window; `device_changed` when its device is not the bound one.
    NewChain { device_changed: bool },
}

impl SurfaceEnd {
    /// The reason `SurfaceLost` carries.
    pub fn reason(self) -> AmmlSurfaceLoss {
        match self {
            Self::WindowGone | Self::NewChain { device_changed: false } => AmmlSurfaceLoss::CHAIN_REPLACED,
            Self::NewDevice | Self::NewChain { device_changed: true } => AmmlSurfaceLoss::DEVICE_CHANGED,
        }
    }
}
