use serde::Serialize;
use ts_rs::TS;

// What the player can do about a check that is not ok.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum CheckActionView {
    Install,
    OpenStore,
}
