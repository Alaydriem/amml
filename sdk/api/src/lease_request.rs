use crate::{InputBlocks, KeyCode, LeaseMode};

/// What a mod asks for in `Input::acquire`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LeaseRequest {
    pub mode: LeaseMode,
    /// The real input the loader blocks while the lease is active.
    pub blocks: InputBlocks,
    /// Real keys that pass a `KEYS` block, for example `KeyCode::ESCAPE`.
    pub passthrough: &'static [KeyCode],
    /// Suspends every other lease while this one is active. One preempt lease at a time.
    pub preempt: bool,
}
