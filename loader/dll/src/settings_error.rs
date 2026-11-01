use std::path::PathBuf;

/// Why the loader config file was not accepted.
#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("{}: {source}", path.display())]
    File { path: PathBuf, source: Box<SettingsError> },
    #[error("cannot read {}: {source}", path.display())]
    Io { path: PathBuf, source: std::io::Error },
    #[error("invalid loader config: {0}")]
    Syntax(String),
    #[error("the loader config takes attributes only, not blocks")]
    Blocks,
    #[error("unknown attribute `{0}` in the loader config")]
    UnknownAttribute(String),
    #[error("`{0}` has the wrong type in the loader config")]
    WrongType(String),
}

impl SettingsError {
    /// This error as found in the file at `path`.
    pub fn in_file(self, path: &std::path::Path) -> Self {
        Self::File { path: path.to_path_buf(), source: Box::new(self) }
    }
}
