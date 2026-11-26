use crate::structs::{ResourceKind, WorldEpoch};

/// One live resource in a store slot, with what binds it.
pub(crate) struct Held<T> {
    pub(crate) kind: ResourceKind,
    pub(crate) owner: usize,
    pub(crate) epoch: WorldEpoch,
    pub(crate) value: T,
}
