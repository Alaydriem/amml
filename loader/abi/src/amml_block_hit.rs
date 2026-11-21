use crate::{AmmlBlockPos, AmmlFace, AmmlVec3};

/// What `raycast` found: `hit` 1 for a block, then the face of the block the ray entered, the block and the point on it
/// where the ray met it. With `hit` 0 the other fields are zero.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AmmlBlockHit {
    pub hit: u8,
    pub face: AmmlFace,
    pub _reserved: [u8; 2],
    pub pos: AmmlBlockPos,
    pub point: AmmlVec3,
}

const _: () = assert!(core::mem::size_of::<AmmlBlockHit>() == 28);
