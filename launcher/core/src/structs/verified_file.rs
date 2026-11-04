use std::path::PathBuf;

// A downloaded file with its SHA-256, checked against the registry when the registry stated one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedFile {
    pub path: PathBuf,
    pub sha256: String,
}
