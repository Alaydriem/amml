use super::DataModeView;
use serde::Serialize;
use ts_rs::TS;

// Where one instance keeps its worlds. `folder` is set for an instance with its own data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct WorldLocationView {
    pub instance: String,
    pub mode: DataModeView,
    pub folder: Option<String>,
}
