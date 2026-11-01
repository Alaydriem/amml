use std::path::PathBuf;

/// The mods root could not be read at all.
#[derive(Debug, thiserror::Error)]
pub enum ModRegistryError {
    #[error("cannot read the mods folder {path}: {source}")]
    Io { path: PathBuf, source: std::io::Error },
}
