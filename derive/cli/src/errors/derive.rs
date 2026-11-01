use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum DeriveError {
    #[error("cannot read {}: {source}", path.display())]
    Io { path: PathBuf, source: std::io::Error },
    #[error("not a valid 64-bit PE image: {0}")]
    NotPe(String),
    #[error("bad pattern token `{token}`")]
    BadPattern { token: String },
}
