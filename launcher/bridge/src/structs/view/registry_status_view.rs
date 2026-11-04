use serde::Serialize;
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum RegistryStatusView {
    Ok,
    Stale,
    Offline,
    Broken,
    Loading,
}
