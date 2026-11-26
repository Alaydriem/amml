use amml_abi::AmmlAabb;

use crate::Vec3;

/// An axis-aligned box in world space, in blocks: the lowest corner, then the highest.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl From<AmmlAabb> for Aabb {
    fn from(value: AmmlAabb) -> Self {
        Self { min: Vec3::from(value.min), max: Vec3::from(value.max) }
    }
}

impl From<Aabb> for AmmlAabb {
    fn from(value: Aabb) -> Self {
        Self { min: value.min.into(), max: value.max.into() }
    }
}
