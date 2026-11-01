use super::OriginalWrite;

/// What the engine must do after a hook was added to or removed from a chain.
///
/// The engine applies `originals` in the order given, then points the target at `first`. The order
/// keeps every slot valid at every moment: a hook's own slot is written before the slot of the
/// hook in front of it, and the target is re-pointed last.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChainChange {
    pub target: u64,
    /// Detour of the first hook, or `None` when the chain is empty and the target must run the original.
    pub first: Option<u64>,
    /// Every slot of the chain, last hook first.
    pub originals: Vec<OriginalWrite>,
}
