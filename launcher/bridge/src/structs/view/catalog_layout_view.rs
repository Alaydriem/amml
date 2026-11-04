use super::CatalogGroupView;
use serde::Serialize;
use ts_rs::TS;

// The shape of one registry's index for one API: the pages are loaded as the player scrolls.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CatalogLayoutView {
    pub registry_id: String,
    pub api: u32,
    pub page_size: u32,
    pub total: u32,
    pub groups: Vec<CatalogGroupView>,
}
