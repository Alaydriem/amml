use crate::{BlockHandle, LightFalloff, Vec3};

/// One held light for `Lighting::set_light_source`: where it is, its level (1 to 15), how it fades and, for the
/// Vibrant Visuals emitter, the block it is (`ItemView::block`). The loader moves the light in quarter-block steps.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LightSource {
    pub position: Vec3,
    pub level: u8,
    pub falloff: LightFalloff,
    pub block: Option<BlockHandle>,
}
