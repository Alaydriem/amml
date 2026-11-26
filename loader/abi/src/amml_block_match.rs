use crate::AmmlBlockHandle;

/// The block `WorldBlockFromStateFn` found: `block` is `AmmlBlockHandle::NONE` when the game knows no block of the
/// name. `exact` is 1 when the block holds every state asked for with the value asked for, and 0 when the game left
/// out a state it does not know or whose value does not fit (the block then has the default value of that state).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AmmlBlockMatch {
    pub block: AmmlBlockHandle,
    pub exact: u8,
    pub _reserved: [u8; 7],
}

const _: () = assert!(core::mem::size_of::<AmmlBlockMatch>() == 16);
