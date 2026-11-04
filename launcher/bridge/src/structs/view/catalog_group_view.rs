use serde::Serialize;
use ts_rs::TS;

// A run of rows that share a first letter. `start` is the index of its first row among the rows for the API.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CatalogGroupView {
    pub id: String,
    pub label: String,
    pub start: u32,
    pub total: u32,
}
