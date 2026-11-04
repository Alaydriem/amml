use super::GameVersion;
use std::path::PathBuf;

// A complete game folder in the version store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionFolder {
    pub version: GameVersion,
    pub path: PathBuf,
    pub files: u64,
    pub bytes: u64,
}
