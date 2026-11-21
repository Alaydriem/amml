use amml_abi::AmmlBlockPos;

/// A block position in the world, in blocks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl From<BlockPos> for AmmlBlockPos {
    fn from(pos: BlockPos) -> Self {
        Self { x: pos.x, y: pos.y, z: pos.z }
    }
}

impl From<AmmlBlockPos> for BlockPos {
    fn from(pos: AmmlBlockPos) -> Self {
        Self { x: pos.x, y: pos.y, z: pos.z }
    }
}
