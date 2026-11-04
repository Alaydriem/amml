use super::FaultKindView;
use serde::Serialize;
use std::collections::BTreeMap;
use ts_rs::TS;

// What went wrong, as facts. The app words it; `code` is the one place an English error text may appear.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct FaultView {
    pub kind: FaultKindView,
    pub code: Option<String>,
    pub values: BTreeMap<String, String>,
    // the Store install was put back before the job ended
    pub restored: bool,
}

impl FaultView {
    pub fn new(kind: FaultKindView, code: Option<String>, values: BTreeMap<String, String>) -> FaultView {
        FaultView { kind, code, values, restored: false }
    }
}
