use crate::AmmlRgba;

/// What `read_columns` found in one column. A column that was not asked for is left as the caller made it.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AmmlColumnSample {
    /// The block type id of the block the walk shows; 0 for none.
    pub shown: u32,
    /// The block type id of the water above `shown`; 0 for none.
    pub water: u32,
    pub shown_color: AmmlRgba,
    pub water_color: AmmlRgba,
    /// The block y of the shown block.
    pub shown_y: i16,
    /// The height of the column from the heightmap, as a block y.
    pub height: i16,
    /// The height of the column from the render heightmap, as a block y.
    pub solid: i16,
    /// The biome id of the column surface; `NO_BIOME` when the loader does not offer `world.biomes` or the biome could
    /// not be read.
    pub biome: u16,
    /// The depth of the water above the shown block; 0 for none, 255 at most.
    pub water_depth: u8,
    /// `SKIPPED`, `RETRY`, `VOID`, `NO_COLOR`.
    pub flags: u8,
    pub _reserved: [u8; 2],
}

impl AmmlColumnSample {
    /// The column was not read: its sub-chunk is still the placeholder, or the chunk has no readable range.
    pub const SKIPPED: u8 = 1;
    /// The walk met a placeholder block: read the column again later.
    pub const RETRY: u8 = 2;
    /// The walk found nothing but air down to the bottom of the chunk.
    pub const VOID: u8 = 4;
    /// No colour is known for what the walk found: a shown block, or a `VOID` column of a request that asked for the
    /// colours. The colours stay zero (alpha 0).
    pub const NO_COLOR: u8 = 8;
    /// The `biome` value of a column with no biome id.
    pub const NO_BIOME: u16 = u16::MAX;
}

const _: () = assert!(core::mem::size_of::<AmmlColumnSample>() == 28);
