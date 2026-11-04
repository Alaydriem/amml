use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AmmlRelease {
    pub version: String,
    pub api_version: u32,
    pub game_versions: Vec<String>,
    pub asset: String,
    pub sha256: String,
    pub size: u64,
}
