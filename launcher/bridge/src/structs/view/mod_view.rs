use super::{ModOriginView, UpdateStatusView};
use serde::Serialize;
use ts_rs::TS;

// A mod of an instance. The AMML row has the id `alaydriem/amml` and the origin `core`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ModView {
    pub id: String,
    pub title: Option<String>,
    pub origin: ModOriginView,
    pub registry_base: Option<String>,
    pub enabled: bool,
    pub installed_version: Option<String>,
    pub available_version: Option<String>,
    pub api: u32,
    pub update: UpdateStatusView,
}
