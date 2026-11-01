/// Layout mirror of the game's `mce::Color` (16 bytes). A fill uses `r`, `g` and `b` and takes its alpha from the
/// call.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct UiColorLayout {
    pub r: f32, // 0x00
    pub g: f32, // 0x04
    pub b: f32, // 0x08
    pub a: f32, // 0x0C
}

impl UiColorLayout {
    /// An opaque color.
    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }
}

const _: () = assert!(size_of::<UiColorLayout>() == 16);
