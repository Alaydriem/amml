use amml_abi::AmmlStatus;

/// Why a call into the loader failed. `From<AmmlStatus>` and `status` are the one mapping between
/// this enum and the ABI status codes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ApiError {
    #[error("the loader cannot serve this on this game build")]
    NotAvailable,
    #[error("the game is not running yet")]
    NotReady,
    #[error("the loader refused the call")]
    Refused,
    #[error("the value is from a world or session that has ended")]
    Stale,
    #[error("the loader returned status {0:?}")]
    Loader(AmmlStatus),
}

impl ApiError {
    /// `Ok` for `AmmlStatus::OK`, otherwise the error for the status.
    pub fn check(status: AmmlStatus) -> Result<(), ApiError> {
        if status.is_ok() { Ok(()) } else { Err(Self::from(status)) }
    }

    /// The status a mod callback reports for this error.
    pub fn status(&self) -> AmmlStatus {
        match self {
            Self::NotAvailable => AmmlStatus::NOT_AVAILABLE,
            Self::NotReady => AmmlStatus::NOT_READY,
            Self::Refused => AmmlStatus::REFUSED,
            Self::Stale => AmmlStatus::STALE,
            Self::Loader(status) => *status,
        }
    }
}

impl From<AmmlStatus> for ApiError {
    fn from(status: AmmlStatus) -> Self {
        match status {
            AmmlStatus::NOT_AVAILABLE => Self::NotAvailable,
            AmmlStatus::NOT_READY => Self::NotReady,
            AmmlStatus::REFUSED => Self::Refused,
            AmmlStatus::STALE => Self::Stale,
            other => Self::Loader(other),
        }
    }
}
