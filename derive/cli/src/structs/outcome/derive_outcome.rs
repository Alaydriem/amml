use amml_data::{BuildData, EntryId};

use super::{Degraded, Failure};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeriveOutcome {
    pub data: BuildData,
    // failures of the entries derived in this run; reused failures are only in `data.failed`
    pub failures: Vec<Failure>,
    // resolved entries with a failed recipe or without a second opinion, in dependency order
    pub degraded: Vec<Degraded>,
    // entries taken from the known data instead of derived, sorted by id
    pub reused: Vec<EntryId>,
}

impl DeriveOutcome {
    /// An outcome in which nothing was reused.
    pub fn new(data: BuildData, failures: Vec<Failure>, degraded: Vec<Degraded>) -> Self {
        Self { data, failures, degraded, reused: Vec::new() }
    }
}
