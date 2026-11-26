use std::collections::{BTreeMap, BTreeSet};

/// The vptr-store starts, the barriers (every RIP-relative `lea` target, a superset of the starts), and for each
/// barrier that code stores as a vptr at any displacement, the set of those displacements.
pub(crate) struct VtableAnchors {
    pub(crate) starts: BTreeSet<u32>,
    pub(crate) barriers: BTreeSet<u32>,
    pub(crate) offsets: BTreeMap<u32, BTreeSet<u64>>,
}
