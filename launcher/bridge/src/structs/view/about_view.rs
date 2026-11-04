use super::RegistryView;
use serde::Serialize;
use ts_rs::TS;

// Facts for the About page and the bug report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AboutView {
    pub launcher_version: String,
    // UTC date, YYYY-MM-DD
    pub built_on: String,
    pub windows_version: Option<String>,
    pub store_package: Option<String>,
    pub store_version: Option<String>,
    pub developer_mode: bool,
    // the official registry as the last refresh saw it
    pub registry: Option<RegistryView>,
    pub logs_dir: String,
}
