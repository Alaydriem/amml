use super::JobStepStateView;
use serde::Serialize;
use ts_rs::TS;

// One step of a job. `id` is a fixed word such as `version-install` or `mod:<id>`; the UI words it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct JobStepView {
    pub id: String,
    pub state: JobStepStateView,
    // how long the step took, once it is done
    #[ts(type = "number | null")]
    pub millis: Option<u64>,
    // the step was skipped because its file was already on disk
    pub cached: bool,
    pub percent: Option<u32>,
}
