use amml_abi::AmmlColumnSample;

use crate::{BlockTypeId, Rgba};

/// What `LevelChunk::read_columns` found in one column of a chunk.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ColumnSample {
    /// The block the walk shows; `NONE` when it shows nothing.
    pub shown: BlockTypeId,
    /// The water above `shown` (transparent water only); `NONE` for none.
    pub water: BlockTypeId,
    pub shown_color: Rgba,
    pub water_color: Rgba,
    /// The block y of the shown block.
    pub shown_y: i16,
    /// The height of the column from the heightmap, as a block y.
    pub height: i16,
    /// The height of the column from the render heightmap, as a block y.
    pub solid: i16,
    /// The biome id of the column surface; `NO_BIOME` without `Area::WORLD_BIOMES`.
    pub biome: u16,
    /// The depth of the water above `shown`; 0 for none, 255 at most.
    pub water_depth: u8,
    /// `SKIPPED`, `RETRY`, `VOID`, `NO_COLOR`.
    pub flags: u8,
}

impl ColumnSample {
    /// The column was not read: its sub-chunk is still the placeholder, or the chunk is not loaded.
    pub const SKIPPED: u8 = AmmlColumnSample::SKIPPED;
    /// The walk met a placeholder block: read the column again later.
    pub const RETRY: u8 = AmmlColumnSample::RETRY;
    /// Nothing but air down to the bottom of the chunk.
    pub const VOID: u8 = AmmlColumnSample::VOID;
    /// No colour is known for what the walk found: a shown block, or a `VOID` column of a request that asked for the
    /// colours. The colours stay zero (alpha 0).
    pub const NO_COLOR: u8 = AmmlColumnSample::NO_COLOR;
    /// The `biome` value of a column with no biome id.
    pub const NO_BIOME: u16 = AmmlColumnSample::NO_BIOME;

    /// Whether the column was not read.
    pub fn skipped(&self) -> bool {
        self.flags & Self::SKIPPED != 0
    }

    /// Whether the column is to be read again later.
    pub fn retry(&self) -> bool {
        self.flags & Self::RETRY != 0
    }
}

impl From<AmmlColumnSample> for ColumnSample {
    fn from(sample: AmmlColumnSample) -> Self {
        Self {
            shown: BlockTypeId(sample.shown),
            water: BlockTypeId(sample.water),
            shown_color: Rgba::from(sample.shown_color),
            water_color: Rgba::from(sample.water_color),
            shown_y: sample.shown_y,
            height: sample.height,
            solid: sample.solid,
            biome: sample.biome,
            water_depth: sample.water_depth,
            flags: sample.flags,
        }
    }
}

impl From<ColumnSample> for AmmlColumnSample {
    fn from(sample: ColumnSample) -> Self {
        Self {
            shown: sample.shown.0,
            water: sample.water.0,
            shown_color: sample.shown_color.into(),
            water_color: sample.water_color.into(),
            shown_y: sample.shown_y,
            height: sample.height,
            solid: sample.solid,
            biome: sample.biome,
            water_depth: sample.water_depth,
            flags: sample.flags,
            _reserved: [0; 2],
        }
    }
}
