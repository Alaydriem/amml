use serde::{Deserialize, Serialize};

use super::CardRelease;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexCard {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub version: String,
    pub size: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    pub core: bool,
    pub default: bool,
    pub updated: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_amml: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub releases: Vec<CardRelease>,
}
