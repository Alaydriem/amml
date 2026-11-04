use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModRelease {
    pub version: String,
    pub api_version: u32,
    pub asset: String,
    pub sha256: String,
    pub size: u64,
    pub published: String,
}
