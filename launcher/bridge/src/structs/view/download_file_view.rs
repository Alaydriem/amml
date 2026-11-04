use super::DownloadFileStateView;
use serde::Serialize;
use ts_rs::TS;

// One file of the downloads folder. `id` is its file name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DownloadFileView {
    pub id: String,
    #[ts(type = "number")]
    pub size: u64,
    pub state: DownloadFileStateView,
    #[ts(type = "number")]
    pub modified: u64,
}
