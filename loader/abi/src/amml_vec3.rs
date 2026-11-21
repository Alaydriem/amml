/// A point or a vector in world space, in blocks: `x` east, `y` up, `z` south.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct AmmlVec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
