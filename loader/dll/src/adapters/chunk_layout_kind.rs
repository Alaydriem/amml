use std::mem::offset_of;

use crate::{ChunkLayout2651, ChunkLayout2660, ChunkOffsets};

/// Which `LevelChunk` mirror the game build has, from the value of the `LevelChunk.mSubChunks` entry. It is the one
/// mirrored member that moves between the builds; a value that matches no mirror selects none.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChunkLayoutKind {
    /// [`ChunkLayout2651`]: `mSubChunks` at 1C8h.
    Build2651,
    /// [`ChunkLayout2660`]: `mSubChunks` at 228h.
    Build2660,
}

impl ChunkLayoutKind {
    /// The mirror whose `mSubChunks` offset is `value`; `None` when no mirror has it.
    pub fn of(value: u64) -> Option<Self> {
        match value {
            ChunkLayout2651::SUB_CHUNKS => Some(Self::Build2651),
            ChunkLayout2660::SUB_CHUNKS => Some(Self::Build2660),
            _ => None,
        }
    }

    /// The offsets the chunk reads use in this layout.
    pub fn offsets(self) -> ChunkOffsets {
        match self {
            Self::Build2651 => ChunkOffsets {
                min: offset_of!(ChunkLayout2651, min),
                max: offset_of!(ChunkLayout2651, max),
                load_state: offset_of!(ChunkLayout2651, load_state),
                sub_chunks: offset_of!(ChunkLayout2651, sub_chunks),
                heightmap: offset_of!(ChunkLayout2651, heightmap),
                render_heightmap: offset_of!(ChunkLayout2651, render_heightmap),
            },
            Self::Build2660 => ChunkOffsets {
                min: offset_of!(ChunkLayout2660, min),
                max: offset_of!(ChunkLayout2660, max),
                load_state: offset_of!(ChunkLayout2660, load_state),
                sub_chunks: offset_of!(ChunkLayout2660, sub_chunks),
                heightmap: offset_of!(ChunkLayout2660, heightmap),
                render_heightmap: offset_of!(ChunkLayout2660, render_heightmap),
            },
        }
    }
}
