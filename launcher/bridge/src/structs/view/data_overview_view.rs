use super::{WorldLocationView, WorldSourceView};
use serde::Serialize;
use ts_rs::TS;

// The shared data folder in figures, where each instance keeps its worlds, and the folders worlds can come from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DataOverviewView {
    pub folder: String,
    pub worlds: u32,
    pub resource_packs: u32,
    pub behavior_packs: u32,
    #[ts(type = "number")]
    pub bytes: u64,
    pub locations: Vec<WorldLocationView>,
    pub sources: Vec<WorldSourceView>,
}
