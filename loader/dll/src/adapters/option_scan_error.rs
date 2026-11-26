/// Why a save tag scan of the option registry found no single option.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum OptionScanError {
    /// An option gave back another id than the one it was asked for in `Option::Impl::mID`: the Impl layout moved.
    #[error("option {id} holds the id {found}")]
    IdMismatch { id: i32, found: i32 },
    /// No option, or more than one, carries the tag.
    #[error("{matches} options carry the tag")]
    Matches { matches: usize },
}

impl OptionScanError {
    /// How many options carried the tag: 0 for an id mismatch.
    pub fn matches(&self) -> usize {
        match self {
            Self::Matches { matches } => *matches,
            Self::IdMismatch { .. } => 0,
        }
    }
}
