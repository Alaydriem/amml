use serde::Serialize;
use ts_rs::TS;

// Bytes of the file that is coming down. `rate` is bytes a second over the last few seconds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ProgressView {
    #[ts(type = "number")]
    pub done: u64,
    #[ts(type = "number")]
    pub total: u64,
    #[ts(type = "number | null")]
    pub rate: Option<u64>,
    #[ts(type = "number | null")]
    pub remaining_seconds: Option<u64>,
    pub file: Option<String>,
    pub mirror: Option<u32>,
}
