/// A block position in the world, in blocks.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct AmmlBlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

const _: () = assert!(core::mem::size_of::<AmmlBlockPos>() == 12);
