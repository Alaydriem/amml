use serde::Serialize;
use ts_rs::TS;

// An instance made by another launcher that this launcher can take over.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AdoptView {
    pub name: String,
    pub game: String,
    pub amml: Option<String>,
    pub mod_count: u32,
}
