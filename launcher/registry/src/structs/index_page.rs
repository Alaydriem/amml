use serde::{Deserialize, Serialize};

use super::IndexCard;

/// `index/api-<n>/<page>.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexPage {
    pub schema: u32,
    pub api_version: u32,
    pub page: u32,
    pub cards: Vec<IndexCard>,
}
