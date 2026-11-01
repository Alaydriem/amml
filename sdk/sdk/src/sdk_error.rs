use amml_abi::AmmlStatus;

/// Why an SDK call failed.
#[derive(Debug, thiserror::Error)]
pub enum SdkError {
    #[error("the loader gave no API table")]
    NullApi,
    #[error("the loader API table is {0} bytes, too small for version 1")]
    TableTooSmall(u32),
    #[error("the loader speaks API version {0}, older than this mod needs")]
    LoaderTooOld(u32),
    #[error("entry `{0}` did not resolve")]
    MissingEntry(String),
    #[error("the loader returned status {0:?}")]
    Status(AmmlStatus),
    #[error("a string from the loader is not valid UTF-8")]
    InvalidString,
    #[error("{0}")]
    Mod(String),
}

impl SdkError {
    /// The status a mod callback reports for this error.
    pub fn status(&self) -> AmmlStatus {
        match self {
            Self::NullApi | Self::InvalidString => AmmlStatus::INVALID_ARGUMENT,
            Self::TableTooSmall(_) | Self::LoaderTooOld(_) => AmmlStatus::VERSION_MISMATCH,
            Self::MissingEntry(_) => AmmlStatus::MISSING_ENTRY,
            Self::Status(status) => *status,
            Self::Mod(_) => AmmlStatus::FAILED,
        }
    }
}
