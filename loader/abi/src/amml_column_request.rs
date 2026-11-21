/// Which columns of a chunk `read_columns` reads, and how. Column `(x, z)` of the chunk is bit `z * 16 + x` of
/// `columns` (word `bit / 64`, bit `bit % 64`).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AmmlColumnRequest {
    /// The block y the walk starts from.
    pub camera_y: i32,
    /// `TRANSPARENT_WATER` and `VANILLA_COLORS`.
    pub flags: u8,
    pub _reserved: [u8; 3],
    /// The columns to read: 256 bits.
    pub columns: [u64; 4],
}

impl AmmlColumnRequest {
    /// Water shows the solid block below it, with the water depth and colour beside it.
    pub const TRANSPARENT_WATER: u8 = 1;
    /// The colours are the game's own map colours.
    pub const VANILLA_COLORS: u8 = 2;
}

const _: () = assert!(core::mem::size_of::<AmmlColumnRequest>() == 40);
