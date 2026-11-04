use super::CardView;
use serde::Serialize;
use ts_rs::TS;

// The hits of one registry. `total` counts every hit; `cards` holds the best ones.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SearchGroupView {
    pub registry_id: String,
    pub total: u32,
    pub cards: Vec<CardView>,
}
