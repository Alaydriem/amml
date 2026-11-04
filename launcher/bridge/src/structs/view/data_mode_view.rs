use serde::Serialize;
use ts_rs::TS;

// Where an instance keeps the game's roaming data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum DataModeView {
    Shared,
    Own,
}
