use crate::{AmmlBlockPos, AmmlSkullPose};

/// The options of a block entity ghost (`AmmlRenderTable::block_entity_create`).
///
/// `chest_pair` is read only when `has_chest_pair` is 1: the position of the other half of a large chest relative to
/// the ghost, one block along x or z at the same height (area `render.chest_pair`). `skull` is read only when `has_skull`
/// is 1 (area `render.skull_pose`). `_reserved` stays zero.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AmmlGhostEntityOptions {
    pub has_chest_pair: u8,
    pub has_skull: u8,
    pub skull: AmmlSkullPose,
    pub _reserved: [u8; 4],
    pub chest_pair: AmmlBlockPos,
}

const _: () = assert!(core::mem::size_of::<AmmlGhostEntityOptions>() == 20);
