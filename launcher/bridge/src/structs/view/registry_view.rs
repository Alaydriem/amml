use super::RegistryStatusView;
use serde::Serialize;
use ts_rs::TS;

// A registry. `id` is the cache key of its base URL.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RegistryView {
    pub id: String,
    pub base: String,
    pub name: Option<String>,
    pub official: bool,
    pub status: RegistryStatusView,
    pub mod_count: Option<u32>,
    #[ts(type = "number | null")]
    pub fetched: Option<u64>,
}
