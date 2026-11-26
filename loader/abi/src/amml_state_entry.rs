use crate::{AmmlStateValue, AmmlStr};

/// One block state by name (such as `facing_direction`) and value, as `WorldBlockFromStateFn` takes them.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AmmlStateEntry {
    pub name: AmmlStr,
    pub value: AmmlStateValue,
}

const _: () = assert!(core::mem::size_of::<AmmlStateEntry>() == 40);
