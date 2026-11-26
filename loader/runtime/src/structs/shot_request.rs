use std::path::PathBuf;

/// A screenshot a mod asked for, handed out by `ShotBook::due`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShotRequest {
    pub id: u64,
    pub owner: usize,
    pub path: PathBuf,
}
