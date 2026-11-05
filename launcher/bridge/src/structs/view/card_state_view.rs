use serde::Serialize;
use ts_rs::TS;

// A catalogue row against the instance the player is adding to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum CardStateView {
    Available,
    Installed,
    Update,
    // the newest release needs a newer AMML than the instance has
    NeedsAmml,
}
