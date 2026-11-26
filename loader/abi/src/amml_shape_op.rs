use crate::{AmmlRgba, AmmlVec3};

/// One shape change of a `shapes_apply` batch (48 bytes). `kind` is `LINE`, `BOX`, `ARROW` or `REMOVE`; `id` is the
/// mod's own shape id (the loader maps it to a game id per mod). A line goes from `from` to `to`; a box has its lowest
/// corner at `from` and its size along each axis in `to`; an arrow goes from `from` to `to` with the head at `to`. A
/// remove reads only `id`. `color` is the shape's colour and alpha.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AmmlShapeOp {
    pub kind: u8,
    pub _reserved: [u8; 7],
    pub id: u64,
    pub from: AmmlVec3,
    pub to: AmmlVec3,
    pub color: AmmlRgba,
    pub _pad: u32,
}

impl AmmlShapeOp {
    /// A line segment.
    pub const LINE: u8 = 1;
    /// A box.
    pub const BOX: u8 = 2;
    /// An arrow.
    pub const ARROW: u8 = 3;
    /// Removes the shape `id`.
    pub const REMOVE: u8 = 4;
}

const _: () = assert!(core::mem::size_of::<AmmlShapeOp>() == 48);
const _: () = assert!(core::mem::offset_of!(AmmlShapeOp, id) == 8);
const _: () = assert!(core::mem::offset_of!(AmmlShapeOp, color) == 40);
