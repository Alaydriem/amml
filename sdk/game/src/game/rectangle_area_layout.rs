/// Layout mirror of the game's `RectangleArea` (16 bytes): the left and right edge, then the top and bottom edge.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct RectangleAreaLayout {
    pub x0: f32, // 0x00 left
    pub x1: f32, // 0x04 right
    pub y0: f32, // 0x08 top
    pub y1: f32, // 0x0C bottom
}

impl RectangleAreaLayout {
    pub fn new(x0: f32, x1: f32, y0: f32, y1: f32) -> Self {
        Self { x0, x1, y0, y1 }
    }
}

const _: () = assert!(size_of::<RectangleAreaLayout>() == 16);
