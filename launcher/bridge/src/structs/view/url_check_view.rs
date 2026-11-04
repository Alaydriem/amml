use serde::Serialize;
use ts_rs::TS;

// What a zip from a URL turned out to be.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UrlCheckView {
    pub id: String,
    pub title: Option<String>,
    pub version: Option<String>,
    pub api: u32,
    pub api_matches: bool,
    // the base URL of the registry that lists this exact zip
    pub matched_registry: Option<String>,
    pub has_checksum: bool,
}
