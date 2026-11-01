/// An `AABB` of the game: an axis-aligned box of six floats, the minimum corner first.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Aabb {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl Aabb {
    /// Whether the box is a point: the minimum equals the maximum on every axis. A block without a collision shape
    /// gives the all-zero box, and a zero-volume box at any other position is empty too.
    pub fn is_empty(&self) -> bool {
        self.min == self.max
    }
}
