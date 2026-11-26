use amml_abi::AmmlVec3;

/// A point or a vector in world space, in blocks: `x` east, `y` up, `z` south.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<Vec3> for AmmlVec3 {
    fn from(value: Vec3) -> Self {
        Self { x: value.x, y: value.y, z: value.z }
    }
}

impl From<AmmlVec3> for Vec3 {
    fn from(value: AmmlVec3) -> Self {
        Self { x: value.x, y: value.y, z: value.z }
    }
}
