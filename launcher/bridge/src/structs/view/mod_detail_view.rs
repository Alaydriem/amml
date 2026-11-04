use super::CardView;
use serde::Serialize;
use ts_rs::TS;

// The card plus what the mod document adds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ModDetailView {
    pub card: CardView,
    pub description: Option<String>,
    pub authors: Vec<String>,
    pub license: String,
    pub source: String,
    pub homepage: Option<String>,
    pub api: u32,
    pub sha256: String,
    // the registry could not be reached and the document is the saved copy
    pub stale: bool,
}
