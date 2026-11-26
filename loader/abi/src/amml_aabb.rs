use crate::AmmlVec3;

/// An axis-aligned box in world space, in blocks: the lowest corner, then the highest.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct AmmlAabb {
    pub min: AmmlVec3,
    pub max: AmmlVec3,
}
