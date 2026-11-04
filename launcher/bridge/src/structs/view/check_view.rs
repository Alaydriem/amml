use super::{CheckActionView, CheckIdView, CheckStateView};
use serde::Serialize;
use ts_rs::TS;

// One line of the system check. Facts only: the UI words the title, the detail and the value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CheckView {
    pub id: CheckIdView,
    pub state: CheckStateView,
    pub package: Option<String>,
    pub version: Option<String>,
    #[ts(type = "number | null")]
    pub free_bytes: Option<u64>,
    pub drive: Option<String>,
    pub action: Option<CheckActionView>,
    // the action asks Windows for admin rights once
    pub elevated: bool,
}
