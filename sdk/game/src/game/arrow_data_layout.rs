use std::mem::offset_of;

use amml_msvc::Optional;

// the game's ArrowDataPayload (24h bytes), one alternative of the shape's extra data
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ArrowDataLayout {
    pub end: Optional<[f32; 3]>,    // 0x00 mEndLocation
    pub head_length: Optional<f32>, // 0x10 mArrowHeadLength
    pub head_radius: Optional<f32>, // 0x18 mArrowHeadRadius
    pub segments: Optional<u8>,     // 0x20 mNumSegments
}

const _: () = assert!(offset_of!(ArrowDataLayout, segments) == 0x20);
const _: () = assert!(size_of::<ArrowDataLayout>() == 0x24);
