use std::mem::offset_of;

use amml_msvc::Optional;

// the game's PrimitiveShapeDataPayload (B8h bytes). The extra data is a std::variant of the shape kinds (NullType,
// Arrow, Text, Box, Line, Sphere, ...): its storage, then the alternative index. `PrimitiveShapeDataPayload.size`
// checks the size
#[repr(C)]
pub struct PrimitiveShapeLayout {
    pub network_id: u64,                    // 0x00 mNetworkId
    pub shape_type: Optional<u8>,           // 0x08 mShapeType (ScriptPrimitiveShapeType)
    pub location: Optional<[f32; 3]>,       // 0x0C mLocation
    pub rotation: Optional<[f32; 3]>,       // 0x1C mRotation
    pub scale: Optional<f32>,               // 0x2C mScale
    pub color: Optional<[f32; 4]>,          // 0x34 mColor (mce::Color r, g, b, a)
    pub time_left: Optional<f32>,           // 0x48 mTimeLeftTotalSec
    pub dimension: Optional<i32>,           // 0x50 mDimensionId
    pub attached_to: Optional<i64>,         // 0x58 mAttachedToId
    pub max_render_distance: Optional<f32>, // 0x68 mMaxRenderDistance
    pub extra: [u64; 8],                    // 0x70 mExtraDataPayload storage
    pub extra_index: u8,                    // 0xB0 mExtraDataPayload alternative index
}

const _: () = assert!(offset_of!(PrimitiveShapeLayout, location) == 0x0C);
const _: () = assert!(offset_of!(PrimitiveShapeLayout, color) == 0x34);
const _: () = assert!(offset_of!(PrimitiveShapeLayout, dimension) == 0x50);
const _: () = assert!(offset_of!(PrimitiveShapeLayout, attached_to) == 0x58);
const _: () = assert!(offset_of!(PrimitiveShapeLayout, extra) == 0x70);
const _: () = assert!(offset_of!(PrimitiveShapeLayout, extra_index) == 0xB0);
const _: () = assert!(size_of::<PrimitiveShapeLayout>() == 0xB8);
