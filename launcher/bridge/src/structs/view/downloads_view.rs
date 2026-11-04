use super::DownloadFileView;
use serde::Serialize;
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DownloadsView {
    pub folder: String,
    #[ts(type = "number")]
    pub used: u64,
    pub files: Vec<DownloadFileView>,
}
