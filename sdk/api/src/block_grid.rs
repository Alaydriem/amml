use crate::{BlockHandle, BlockPos};

/// A box of blocks for `BlockMesher::build`: `dims` cells on x, y and z, at most `BlockMesher::MAX_DIM` on each axis, with
/// one cell of border on each side. The border cells are the neighbours of the inner cells (a fence or a glass pane joins
/// them) but are not drawn. `origin` is the world position of cell (0, 0, 0), and the mesh positions are relative to it.
/// `cells` holds `dims[0] * dims[1] * dims[2]` palette indices, x first, then z, then y (cell (x, y, z) at
/// `(y * dims[2] + z) * dims[0] + x`); `EMPTY` is a cell with no block: the loader puts air there, so a ghost cell does not join the world. `palette` holds this
/// mod's block handles.
#[derive(Clone, Copy, Debug)]
pub struct BlockGrid<'a> {
    pub dims: [u32; 3],
    pub origin: BlockPos,
    pub cells: &'a [u16],
    pub palette: &'a [BlockHandle],
}

impl BlockGrid<'_> {
    /// The palette index of an empty cell.
    pub const EMPTY: u16 = u16::MAX;
}
