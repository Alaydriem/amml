use std::path::PathBuf;

use amml_abi::AmmlStatus;

/// Why a mod did not load or enable.
#[derive(Debug, thiserror::Error)]
pub enum ModLoadError {
    #[error("cannot load {path}: {source}")]
    Library { path: PathBuf, source: std::io::Error },
    #[error("the DLL has no export `{0}`")]
    MissingExport(&'static str),
    #[error("the mod needs API version {needs} and this loader has version {loader}")]
    VersionMismatch { needs: u32, loader: u32 },
    #[error("`{call}` returned status {status:?}")]
    Call { call: &'static str, status: AmmlStatus },
    #[error("`amml_mod_load` returned OK but left the info block empty")]
    EmptyInfo,
    #[error("a folder path of the mod is not valid UTF-8")]
    PathNotUtf8,
    #[error("the loader already holds {0} mods")]
    TooMany(usize),
}

impl ModLoadError {
    /// The status that describes this error.
    pub fn status(&self) -> AmmlStatus {
        match self {
            Self::VersionMismatch { .. } => AmmlStatus::VERSION_MISMATCH,
            Self::Call { status, .. } => *status,
            Self::Library { .. } | Self::MissingExport(_) | Self::EmptyInfo | Self::PathNotUtf8 | Self::TooMany(_) => {
                AmmlStatus::FAILED
            }
        }
    }
}
