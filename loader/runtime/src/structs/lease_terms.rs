use amml_abi::{AmmlInputBlocks, AmmlLeaseMode};

/// What a mod asks for when it takes an input lease.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeaseTerms {
    pub mode: AmmlLeaseMode,
    pub blocks: AmmlInputBlocks,
    /// Real keys that pass a `KEYS` block.
    pub passthrough: Vec<u16>,
    /// Suspends every other lease while this one is active.
    pub preempt: bool,
}
