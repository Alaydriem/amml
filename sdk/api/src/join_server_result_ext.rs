use amml_abi::JoinServerResult;

use crate::{JoinId, JoinOutcome};

/// Typed reads of a `JoinServerResult`.
pub trait JoinServerResultExt {
    /// The join this answer is for.
    fn join_id(&self) -> JoinId;
    /// How the game answered.
    fn outcome(&self) -> JoinOutcome;
}

impl JoinServerResultExt for JoinServerResult {
    fn join_id(&self) -> JoinId {
        JoinId(self.join)
    }

    fn outcome(&self) -> JoinOutcome {
        JoinOutcome::from(self.outcome)
    }
}
