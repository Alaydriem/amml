use crate::testing::FakeItem;

/// The inventory of the local player a `FakeGame` serves after `set_inventory`: what `LocalPlayer::held_item`,
/// `Inventory::selected_slot`, `Inventory::slot` and `Inventory::slots` give. `held` is the main-hand stack (`None` is
/// an empty hand); `selected_slot` `None` is a read the loader cannot make; `slots` are the 36 slots (0 to 8 the
/// hotbar, 9 to 35 the main inventory), `None` for an empty slot. The reads also need a client with a local player
/// and `inventory.read` offered; the slot reads need version 3 (`Area::INVENTORY_READ.at(3)`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FakeInventory {
    pub held: Option<FakeItem>,
    pub selected_slot: Option<u8>,
    pub slots: [Option<FakeItem>; 36],
}

impl Default for FakeInventory {
    fn default() -> Self {
        Self { held: None, selected_slot: None, slots: [None; 36] }
    }
}
