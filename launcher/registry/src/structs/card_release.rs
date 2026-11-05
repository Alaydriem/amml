use serde::{Deserialize, Serialize};

/// One release of a card's API. An index card lists these only when some release needs a minimum AMML version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CardRelease {
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_amml: Option<String>,
    pub size: u64,
    pub updated: String,
}
