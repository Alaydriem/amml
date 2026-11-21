use amml_abi::AmmlSurfaceSample;

use crate::BlockPos;

/// One surface that `BlockSource::scan_surfaces` found: the block that gives the support, and the raw light of the block
/// above it (0 to 15 each, with no sky darken).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SurfaceSample {
    pub pos: BlockPos,
    pub block_light: u8,
    pub sky_light: u8,
}

impl From<AmmlSurfaceSample> for SurfaceSample {
    fn from(sample: AmmlSurfaceSample) -> Self {
        Self { pos: sample.pos.into(), block_light: sample.block_light, sky_light: sample.sky_light }
    }
}
