use amml_abi::AmmlSurfaceLoss;

/// Why a draw surface was lost (`SurfaceLost`). A value this crate does not know is `Other`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurfaceLoss {
    /// The render device was removed, reset or replaced.
    DeviceChanged,
    /// The swap chain of the bound window was replaced.
    ChainReplaced,
    Other,
}

impl From<AmmlSurfaceLoss> for SurfaceLoss {
    fn from(loss: AmmlSurfaceLoss) -> Self {
        match loss {
            AmmlSurfaceLoss::DEVICE_CHANGED => Self::DeviceChanged,
            AmmlSurfaceLoss::CHAIN_REPLACED => Self::ChainReplaced,
            _ => Self::Other,
        }
    }
}
