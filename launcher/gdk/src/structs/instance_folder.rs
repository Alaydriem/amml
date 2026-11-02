use super::GameVersion;
use std::path::PathBuf;

// A loose game folder a player starts directly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstanceFolder {
    pub name: String,
    pub version: GameVersion,
    pub path: PathBuf,
}
