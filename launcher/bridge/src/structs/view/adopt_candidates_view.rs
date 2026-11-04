use super::{AdoptView, WorldSourceView};
use serde::Serialize;
use ts_rs::TS;

// What the first-run adopt step offers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AdoptCandidatesView {
    pub instances: Vec<AdoptView>,
    pub sources: Vec<WorldSourceView>,
}
