use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiSummary {
    pub mods: u32,
    pub pages: u32,
}
