use crate::AmmlStr;

/// The value of one block state that `WorldBlockFromStateFn` looks up: an int (`INT`, in `int`), a bool (`BOOL`,
/// `int` 0 or 1) or a text (`TEXT`, in `text`). The fields a kind does not use are zero.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AmmlStateValue {
    pub kind: u8,
    pub _reserved: [u8; 3],
    pub int: i32,
    pub text: AmmlStr,
}

impl AmmlStateValue {
    pub const INT: u8 = 1;
    pub const BOOL: u8 = 2;
    pub const TEXT: u8 = 3;
}

const _: () = assert!(core::mem::size_of::<AmmlStateValue>() == 24);
