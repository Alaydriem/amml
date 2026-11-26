/// The pose 70.26 `SkullBlockRenderer::render` gives a skull for one face of its block: the offset added to the
/// position (x, y, z) and the turn about +Y in radians.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SkullFace {
    pub offset: [f32; 3],
    pub angle: f32,
}
