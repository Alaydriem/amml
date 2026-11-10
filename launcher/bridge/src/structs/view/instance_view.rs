use super::DataModeView;
use serde::Serialize;
use ts_rs::TS;

// An instance as the list and the detail page need it. Times are Unix milliseconds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct InstanceView {
    pub name: String,
    pub folder: String,
    pub game: String,
    pub preview: bool,
    pub amml_version: Option<String>,
    // the AMML is a local build (`install`), not a core release
    pub amml_local: bool,
    pub amml_installed: bool,
    pub mod_count: u32,
    pub data_mode: DataModeView,
    pub data_dir: Option<String>,
    pub managed: bool,
    #[ts(type = "number | null")]
    pub running_since: Option<u64>,
    #[ts(type = "number | null")]
    pub last_played: Option<u64>,
    pub update_count: u32,
}
