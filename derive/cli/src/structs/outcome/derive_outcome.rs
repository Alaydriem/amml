use amml_data::BuildData;

use super::{Degraded, Failure};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeriveOutcome {
    pub data: BuildData,
    pub failures: Vec<Failure>,
    // resolved entries with a failed recipe or without a second opinion, in dependency order
    pub degraded: Vec<Degraded>,
}

impl DeriveOutcome {
    pub fn new(data: BuildData, failures: Vec<Failure>, degraded: Vec<Degraded>) -> Self {
        Self { data, failures, degraded }
    }
}
