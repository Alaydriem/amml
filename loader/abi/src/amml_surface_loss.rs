/// Why the draw surface was lost.
///
/// A plain integer so that a value from the other side of the boundary can never be invalid.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AmmlSurfaceLoss(pub u8);

impl AmmlSurfaceLoss {
    /// The render device was removed, reset or replaced.
    pub const DEVICE_CHANGED: Self = Self(0);
    /// The swap chain of the bound window was replaced.
    pub const CHAIN_REPLACED: Self = Self(1);
}
