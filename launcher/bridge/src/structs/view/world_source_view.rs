use serde::Serialize;
use ts_rs::TS;

// A folder of worlds the player may copy into the shared data folder. `id` is `appdata` or an instance name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct WorldSourceView {
    pub id: String,
    pub label: String,
    pub path: String,
    pub world_count: u32,
    #[ts(type = "number | null")]
    pub modified: Option<u64>,
}
