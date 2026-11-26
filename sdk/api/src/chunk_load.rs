use amml_abi::AmmlChunkLoad;

/// How much of a `LevelChunk` the client has.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChunkLoad {
    /// The chunk is not fully loaded: no column can be read.
    NotLoaded,
    /// The chunk is loaded, but its top sub-chunk is still the client-request placeholder: its columns read as
    /// skipped until the client has the blocks.
    TopPlaceholder,
    /// The chunk is loaded and its blocks can be read.
    Loaded,
}

impl ChunkLoad {
    /// The state of an ABI value; `None` for a value this crate does not know.
    pub(crate) fn from_abi(value: AmmlChunkLoad) -> Option<Self> {
        match value {
            AmmlChunkLoad::NOT_LOADED => Some(Self::NotLoaded),
            AmmlChunkLoad::TOP_PLACEHOLDER => Some(Self::TopPlaceholder),
            AmmlChunkLoad::LOADED => Some(Self::Loaded),
            _ => None,
        }
    }

    #[cfg(feature = "test")]
    pub(crate) fn to_abi(self) -> AmmlChunkLoad {
        match self {
            Self::NotLoaded => AmmlChunkLoad::NOT_LOADED,
            Self::TopPlaceholder => AmmlChunkLoad::TOP_PLACEHOLDER,
            Self::Loaded => AmmlChunkLoad::LOADED,
        }
    }
}
