use serde::Serialize;
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SetupView {
    // the settings file does not exist yet, so this is the first run
    pub needed: bool,
}
