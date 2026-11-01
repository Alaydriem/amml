use amml_abi::AmmlStatus;
use amml_runtime::ChainError;

/// Why the hook engine refused or failed a request.
#[derive(Debug, thiserror::Error)]
pub enum HookError {
    #[error("invalid argument: {0}")]
    InvalidArgument(&'static str),
    #[error(transparent)]
    Chain(#[from] ChainError),
    #[error("detour failed: {0}")]
    Detour(#[from] retour::Error),
    #[error("executable memory for the thunk: {0}")]
    Thunk(#[from] region::Error),
}

impl HookError {
    /// The status the C ABI reports for this error.
    pub fn status(&self) -> AmmlStatus {
        match self {
            Self::InvalidArgument(_)
            | Self::Chain(
                ChainError::DuplicateHandle(_)
                | ChainError::UnknownHandle(_)
                | ChainError::DuplicateSlot(_)
                | ChainError::DuplicateDetour(_),
            ) => AmmlStatus::INVALID_ARGUMENT,
            Self::Chain(ChainError::Full(_)) => AmmlStatus::FAILED,
            Self::Detour(_) | Self::Thunk(_) => AmmlStatus::HOOK_FAILED,
        }
    }
}
