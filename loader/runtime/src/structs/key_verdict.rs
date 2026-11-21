use crate::structs::ResourceId;

/// What the key table says about one real key feed, before the listeners run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyVerdict {
    /// Whether the loader raises a `KeyInput` for this feed.
    pub raise: bool,
    /// Whether this is a down of a key that is already down.
    pub repeat: bool,
    /// The leases whose injected hold of the key this real up ended, in the order they pressed it.
    pub ended: Vec<ResourceId>,
}
