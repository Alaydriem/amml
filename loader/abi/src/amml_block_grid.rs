use crate::{AmmlBlockHandle, AmmlBlockPos};

/// A box of blocks a mod asks the block mesher to build: `dims` cells on x, y and z, with one cell of border on each side
/// that is read as the neighbours of the inner cells but not drawn. `origin` is the world position of cell (0, 0, 0).
/// `cells` holds `cell_count` palette indices, x first, then z, then y (cell (x, y, z) at `(y * dims[2] + z) * dims[0] +
/// x`); `u16::MAX` is an empty cell. `palette` holds `palette_len` block handles of the calling mod.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AmmlBlockGrid {
    pub dims: [u32; 3],
    pub origin: AmmlBlockPos,
    pub cells: *const u16,
    pub cell_count: u32,
    pub palette: *const AmmlBlockHandle,
    pub palette_len: u32,
}

impl AmmlBlockGrid {
    /// The palette index of an empty cell.
    pub const EMPTY: u16 = u16::MAX;
}

const _: () = assert!(core::mem::size_of::<AmmlBlockGrid>() == 56);
