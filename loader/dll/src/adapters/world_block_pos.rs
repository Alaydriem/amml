/// A `BlockPos` of the game: a block in the world, three ints.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct WorldBlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

const _: () = assert!(size_of::<WorldBlockPos>() == 12);
