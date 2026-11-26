use amml_abi::AmmlSlotAt;

use crate::Collection;

/// One slot of a container screen: a collection and an index in it, counted from 0.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SlotAt {
    pub collection: Collection,
    pub index: i32,
}

impl From<AmmlSlotAt> for SlotAt {
    fn from(slot: AmmlSlotAt) -> Self {
        Self { collection: Collection::from(slot.collection), index: slot.index }
    }
}

impl From<SlotAt> for AmmlSlotAt {
    fn from(slot: SlotAt) -> Self {
        Self { collection: slot.collection.raw(), index: slot.index }
    }
}
