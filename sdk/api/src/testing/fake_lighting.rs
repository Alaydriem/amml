use std::cell::{Cell, RefCell};

use amml_abi::{AmmlBlockHandle, AmmlLightSource};

use crate::testing::FakeItem;

/// The lighting side of a `FakeGame`: the off-hand stack, the items that place a block (with its light
/// emission), the last light the mod set and how many set calls it made, and whether set calls are refused.
#[derive(Default)]
pub(crate) struct FakeLighting {
    off_hand: Cell<Option<FakeItem>>,
    item_blocks: RefCell<Vec<(&'static str, u8)>>,
    light: Cell<Option<AmmlLightSource>>,
    calls: Cell<usize>,
    refused: Cell<bool>,
}

impl FakeLighting {
    pub(crate) fn off_hand(&self) -> Option<FakeItem> {
        self.off_hand.get()
    }

    pub(crate) fn set_off_hand(&self, item: Option<FakeItem>) {
        self.off_hand.set(item);
    }

    /// Items named `type_name` place a block that gives `emission`.
    pub(crate) fn set_item_block(&self, type_name: &'static str, emission: u8) {
        let mut blocks = self.item_blocks.borrow_mut();
        blocks.retain(|(name, _)| *name != type_name);
        blocks.push((type_name, emission));
    }

    /// The block handle and emission of `item` in world `epoch`: the handle holds the epoch and the item's place in
    /// the block list, so a handle of another world is stale. `NONE` for an item that places no block.
    pub(crate) fn block_of(&self, item: &FakeItem, epoch: u64) -> (AmmlBlockHandle, u8) {
        let blocks = self.item_blocks.borrow();
        match blocks.iter().position(|(name, _)| *name == item.type_name) {
            Some(index) => (AmmlBlockHandle((epoch << 16) | (index as u64 + 1)), blocks[index].1),
            None => (AmmlBlockHandle::NONE, 0),
        }
    }

    /// Whether `handle` is a handle of world `epoch`.
    pub(crate) fn is_current(&self, handle: AmmlBlockHandle, epoch: u64) -> bool {
        let index = handle.0 & 0xFFFF;
        handle.0 >> 16 == epoch && index >= 1 && (index as usize) <= self.item_blocks.borrow().len()
    }

    pub(crate) fn set_light(&self, light: Option<AmmlLightSource>) {
        self.calls.set(self.calls.get() + 1);
        self.light.set(light);
    }

    pub(crate) fn light(&self) -> Option<AmmlLightSource> {
        self.light.get()
    }

    pub(crate) fn calls(&self) -> usize {
        self.calls.get()
    }

    pub(crate) fn refused(&self) -> bool {
        self.refused.get()
    }

    pub(crate) fn set_refused(&self, refused: bool) {
        self.refused.set(refused);
    }
}
