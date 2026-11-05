use super::{CardStateView, NeedsAmmlView};
use serde::Serialize;
use ts_rs::TS;

// One row of a registry index page. `icon_url` is the registry base plus the card's relative icon path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CardView {
    pub id: String,
    pub registry_id: String,
    pub title: String,
    pub summary: String,
    pub tags: Vec<String>,
    pub version: String,
    #[ts(type = "number")]
    pub size: u64,
    pub icon_url: Option<String>,
    pub state: CardStateView,
    // set when `state` is `NeedsAmml`
    pub needs_amml: Option<NeedsAmmlView>,
    pub updated: String,
}
