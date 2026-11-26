use std::mem::offset_of;

use amml_msvc::Optional;

/// The game's `ArrowDataPayload` (24h bytes), the arrow alternative of a shape payload's extra data. Read from the exe
/// on 26.63 and 70.26: the arrow's payload function (vtable slot 2: 0xb8e7090 / 0x10f16c10) tests the engaged byte at
/// +0Ch, then reads the Vec3 at +0; +14h then the float +10h; +1Ch then the float +18h; +21h then the byte +20h. Which
/// of +10h and +18h is the head length and which the head radius is not proven from the exe: the order is the one the
/// 26.63 freecam mod used, which drew the arrows it wanted in game. The order in which the game's script registration
/// names the arrow members ("Arrow Head Length" before "Arrow Head Radius") is not proof: the same registration names
/// the payload's own members out of their layout order (survey section 8.12).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ArrowDataLayout {
    pub end: Optional<[f32; 3]>,    // 0x00 mEndLocation
    pub head_length: Optional<f32>, // 0x10 mArrowHeadLength
    pub head_radius: Optional<f32>, // 0x18 mArrowHeadRadius
    pub segments: Optional<u8>,     // 0x20 mNumSegments
}

impl ArrowDataLayout {
    /// The arrow head the 26.63 freecam mod drew.
    pub const HEAD_LENGTH: f32 = 0.4;
    pub const HEAD_RADIUS: f32 = 0.2;
    pub const HEAD_SEGMENTS: u8 = 8;

    /// The arrow data of an arrow that ends at `end`, with the loader's head.
    pub fn new(end: [f32; 3]) -> Self {
        Self {
            end: Optional::some(end),
            head_length: Optional::some(Self::HEAD_LENGTH),
            head_radius: Optional::some(Self::HEAD_RADIUS),
            segments: Optional::some(Self::HEAD_SEGMENTS),
        }
    }
}

const _: () = assert!(offset_of!(ArrowDataLayout, head_length) == 0x10);
const _: () = assert!(offset_of!(ArrowDataLayout, head_radius) == 0x18);
const _: () = assert!(offset_of!(ArrowDataLayout, segments) == 0x20);
const _: () = assert!(size_of::<ArrowDataLayout>() == 0x24);
