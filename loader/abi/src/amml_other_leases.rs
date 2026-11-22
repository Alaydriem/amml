use crate::AmmlInputBlocks;

/// The active input leases of the mods other than the caller (`InputHeldByOthersFn`): how many, and the union of the
/// real input they block.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AmmlOtherLeases {
    pub leases: u32,
    pub blocks: AmmlInputBlocks,
}

const _: () = assert!(core::mem::size_of::<AmmlOtherLeases>() == 8);
