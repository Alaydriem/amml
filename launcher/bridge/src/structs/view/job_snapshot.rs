use super::{FaultView, JobStateView, JobStepView, ProgressView};
use serde::Serialize;
use ts_rs::TS;

// The job as it stands now. A screen that opens mid-job reads this and has the step and bytes at once.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct JobSnapshot {
    #[ts(type = "number")]
    pub id: u64,
    // the kind as a word: `create-instance`, `update-amml`, `install-mods`, ...
    pub kind: String,
    // the instance, the source folder or the first adopted name the job is about
    pub title_name: String,
    // the game version the job is making, when it makes one
    pub version: Option<String>,
    pub state: JobStateView,
    pub steps: Vec<JobStepView>,
    pub step_number: u32,
    pub step_total: u32,
    pub progress: Option<ProgressView>,
    pub fault: Option<FaultView>,
    #[ts(type = "number")]
    pub started: u64,
}
