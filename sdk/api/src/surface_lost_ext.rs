use amml_abi::SurfaceLost;

use crate::SurfaceLoss;

/// Typed reads of a `SurfaceLost`.
pub trait SurfaceLostExt {
    fn reason(&self) -> SurfaceLoss;
}

impl SurfaceLostExt for SurfaceLost {
    fn reason(&self) -> SurfaceLoss {
        SurfaceLoss::from(self.reason)
    }
}
