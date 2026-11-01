use amml_data::EntryId;

use super::FailureReason;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Failure {
    pub id: EntryId,
    pub reason: FailureReason,
}

impl Failure {
    pub fn new(id: EntryId, reason: FailureReason) -> Self {
        Self { id, reason }
    }
}
