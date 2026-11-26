use std::mem::offset_of;

use amml_msvc::Optional;
use amml_runtime::ShapeKind;

use crate::ArrowDataLayout;

/// The game's `PrimitiveShapeDataPayload` (B8h bytes), one element of `PrimitiveShapesPacket::mShapes`.
/// `PrimitiveShapeDataPayload.size` checks the size. The extra data is a `std::variant` of the shape kinds (NullType,
/// Arrow, Text, Box, Line, Sphere, ...): its storage, then the alternative index at B0h.
///
/// Read from the exe on 26.63 and 70.26: the shape system that the `PrimitiveShapesPacket` handler calls steps through
/// the vector by B8h, compares each shape's game id with the `u64` at +0, and tests the shape type's engaged byte (+9)
/// against 1: a payload with an engaged type updates the shape of that id (slot 2 of the shape object) or makes a new
/// one through the shape factory; a payload without one removes the shape of that id, and is nothing when no shape has
/// that id. The factory switches on the type byte +8 (line 0, box 1, arrow 5). Slot 2 of each shape object tests the
/// alternative index at B0h (line 4, box 3, arrow 1) before it reads the extra data at 70h: the line's and the box's
/// Vec3 at 70h, the arrow's `ArrowDataLayout`. The members from +0Ch to +6Ch are read, each with its engaged byte, by
/// the base shape apply (26.63 inlined at the head of slot 2, 0xb8dd290; 70.26 0x10f0b260), at the offsets below.
#[repr(C)]
#[derive(Clone, Copy)]
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

impl PrimitiveShapeLayout {
    /// The size the `PrimitiveShapeDataPayload.size` entry must have.
    pub const SIZE: u64 = size_of::<PrimitiveShapeLayout>() as u64;
    /// `ScriptModuleMinecraft::ScriptPrimitiveShapeType` values the shape factory makes.
    pub const SHAPE_LINE: u8 = 0;
    pub const SHAPE_BOX: u8 = 1;
    pub const SHAPE_ARROW: u8 = 5;
    /// The alternative index of each kind's extra data.
    pub const EXTRA_ARROW: u8 = 1;
    pub const EXTRA_BOX: u8 = 3;
    pub const EXTRA_LINE: u8 = 4;

    /// Writes the remove of the shape `network_id` into the zeroed payload at `payload`: the id only, every option
    /// disengaged.
    ///
    /// # Safety
    /// `payload` points to `SIZE` zeroed, writable bytes, aligned to 1 or more.
    pub unsafe fn write_remove(payload: *mut Self, network_id: u64) {
        // SAFETY: the caller guarantees the bytes; the payload may be only byte aligned, so the write is unaligned.
        unsafe { (&raw mut (*payload).network_id).write_unaligned(network_id) };
    }

    /// Writes the shape `network_id` of `kind` into the zeroed payload at `payload`: its type, `from` as the location,
    /// no rotation, scale 1, `color` (r, g, b, a in 0..1), `dimension`, and the extra data of the kind with `to` (the
    /// line's end, the box's size, the arrow's end).
    ///
    /// # Safety
    /// As `write_remove`.
    pub unsafe fn write_draw(
        payload: *mut Self,
        network_id: u64,
        kind: ShapeKind,
        [from, to]: [[f32; 3]; 2],
        color: [f32; 4],
        dimension: i32,
    ) {
        let (shape_type, extra_index) = match kind {
            ShapeKind::Line => (Self::SHAPE_LINE, Self::EXTRA_LINE),
            ShapeKind::Box => (Self::SHAPE_BOX, Self::EXTRA_BOX),
            ShapeKind::Arrow => (Self::SHAPE_ARROW, Self::EXTRA_ARROW),
        };
        // SAFETY: the caller guarantees `SIZE` zeroed, writable bytes laid out as the mirror; every member is trivially
        // copyable, and each is written unaligned. Untouched bytes (the disengaged options, the extra storage past the
        // kind's data) stay zero.
        unsafe {
            (&raw mut (*payload).network_id).write_unaligned(network_id);
            (&raw mut (*payload).shape_type).write_unaligned(Optional::some(shape_type));
            (&raw mut (*payload).location).write_unaligned(Optional::some(from));
            (&raw mut (*payload).rotation).write_unaligned(Optional::some([0.0; 3]));
            (&raw mut (*payload).scale).write_unaligned(Optional::some(1.0));
            (&raw mut (*payload).color).write_unaligned(Optional::some(color));
            (&raw mut (*payload).dimension).write_unaligned(Optional::some(dimension));
            (&raw mut (*payload).extra_index).write_unaligned(extra_index);
            let extra = (&raw mut (*payload).extra).cast::<u8>();
            match kind {
                ShapeKind::Arrow => extra.cast::<ArrowDataLayout>().write_unaligned(ArrowDataLayout::new(to)),
                ShapeKind::Line | ShapeKind::Box => extra.cast::<[f32; 3]>().write_unaligned(to),
            }
        }
    }
}

const _: () = assert!(offset_of!(PrimitiveShapeLayout, shape_type) == 0x08);
const _: () = assert!(offset_of!(PrimitiveShapeLayout, location) == 0x0C);
const _: () = assert!(offset_of!(PrimitiveShapeLayout, rotation) == 0x1C);
const _: () = assert!(offset_of!(PrimitiveShapeLayout, scale) == 0x2C);
const _: () = assert!(offset_of!(PrimitiveShapeLayout, color) == 0x34);
const _: () = assert!(offset_of!(PrimitiveShapeLayout, time_left) == 0x48);
const _: () = assert!(offset_of!(PrimitiveShapeLayout, dimension) == 0x50);
const _: () = assert!(offset_of!(PrimitiveShapeLayout, attached_to) == 0x58);
const _: () = assert!(offset_of!(PrimitiveShapeLayout, max_render_distance) == 0x68);
const _: () = assert!(offset_of!(PrimitiveShapeLayout, extra) == 0x70);
const _: () = assert!(offset_of!(PrimitiveShapeLayout, extra_index) == 0xB0);
const _: () = assert!(size_of::<PrimitiveShapeLayout>() == 0xB8);
const _: () = assert!(size_of::<ArrowDataLayout>() <= size_of::<[u64; 8]>());
