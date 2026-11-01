use crate::game::{LevelChunkLayout2651, LevelChunkLayout2660};
use crate::{EntryId, GameEntries};

/// Which `LevelChunk` mirror the game build has, from the value of the `LevelChunk.mSubChunks` entry. It is the one
/// mirrored member that moves between the builds; a value that matches no mirror selects none.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LevelChunkLayoutKind {
    /// [`LevelChunkLayout2651`].
    Build2651,
    /// [`LevelChunkLayout2660`].
    Build2660,
}

impl LevelChunkLayoutKind {
    /// The mirror whose `mSubChunks` offset is the entry value; `None` when the entry is off or no mirror has it.
    pub fn of(entries: &GameEntries) -> Option<Self> {
        match entries.get(EntryId::LEVEL_CHUNK_SUB_CHUNKS)? {
            LevelChunkLayout2651::SUB_CHUNKS => Some(Self::Build2651),
            LevelChunkLayout2660::SUB_CHUNKS => Some(Self::Build2660),
            _ => None,
        }
    }
}
