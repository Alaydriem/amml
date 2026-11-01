use std::path::PathBuf;

use amml_runtime::ModRegistryError;

use crate::{LoaderError, SettingsError};

/// Why host startup stopped.
#[derive(Debug, thiserror::Error)]
pub enum HostError {
    #[error("cannot find the path of the game executable")]
    NoExePath,
    #[error("cannot create {path}: {source}")]
    Folder { path: PathBuf, source: std::io::Error },
    #[error(transparent)]
    Settings(#[from] SettingsError),
    #[error("cannot read the mods folder: {0}")]
    Mods(#[from] ModRegistryError),
    #[error(transparent)]
    Loader(#[from] LoaderError),
}
