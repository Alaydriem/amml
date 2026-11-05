use std::collections::BTreeSet;

/// The vptr-store starts, and the barriers (every RIP-relative `lea` target, a superset of the starts).
pub(crate) struct VtableAnchors {
    pub(crate) starts: BTreeSet<u32>,
    pub(crate) barriers: BTreeSet<u32>,
}
