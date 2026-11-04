use serde::Serialize;
use ts_rs::TS;

// `kept` is the package a restore needs, `unfinished` a partial download, `finished` any other file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum DownloadFileStateView {
    Kept,
    Unfinished,
    Finished,
}
