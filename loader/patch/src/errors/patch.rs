use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum PatchError {
    #[error("the file already imports {0}")]
    AlreadyPatched(String),
    #[error("the section table has no room for another section header")]
    NoHeaderRoom,
    #[error("not a patchable PE file: {0}")]
    Malformed(String),
    #[error("the new import does not fit in a PE image: {0}")]
    TooLarge(String),
    #[error("io error on {}: {source}", path.display())]
    Io { path: PathBuf, source: std::io::Error },
}
