use crate::structs::InjectedOp;

/// What one release of a lease ended.
#[derive(Clone, Debug, PartialEq)]
pub struct ReleaseOutcome {
    /// The ups of the holds this release ended, queued for the next frame.
    pub ups: Vec<InjectedOp>,
    /// The lease held a key or button (queued or applied) or the move intent before the release.
    pub had_input: bool,
}
