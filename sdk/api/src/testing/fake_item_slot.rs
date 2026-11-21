use amml_abi::AmmlItemDraw;

use crate::testing::FakeEntry;

pub(crate) struct FakeItemSlot {
    pub(crate) entry: FakeEntry,
    pub(crate) draw: AmmlItemDraw,
    pub(crate) nested: u64,
    pub(crate) taken: bool,
}
