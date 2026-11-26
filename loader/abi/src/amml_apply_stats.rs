/// How a `shapes_apply` call went: the number of shape payloads handed to the game (a remove counts one, a kind change
/// counts two), and the time the call took in microseconds.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AmmlApplyStats {
    pub delivered: u32,
    pub elapsed_us: u32,
}

const _: () = assert!(core::mem::size_of::<AmmlApplyStats>() == 8);
