/// A size in pixels.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AmmlSize {
    pub width: u32,
    pub height: u32,
}

const _: () = assert!(core::mem::size_of::<AmmlSize>() == 8);
