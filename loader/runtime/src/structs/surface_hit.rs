/// One surface found by `SurfaceRule`: the block y of the block that gives the support, and the raw light of the
/// block above it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SurfaceHit {
    pub y: i32,
    pub block_light: u8,
    pub sky_light: u8,
}
