use amml_runtime::CreativeEntry;

use crate::{
    BlockStateAdapter, CreativeItemEntryLayout, CreativeRegistry, InventoryAdapter, ObjectRead, OwnedItemStack,
    PickCalls, PickUndo,
};

/// The game side of the creative pick: the creative entry of the item of a block, and the stack a pick shows in a
/// hotbar slot of the local player. Built for one call or one tick; it keeps nothing. Game thread only.
pub struct CreativePick<'a> {
    calls: &'a PickCalls,
    net_id_at: usize,
}

impl<'a> CreativePick<'a> {
    // `ItemStackBase::getMaxStackSize` gives this for a stack with no item.
    const NO_ITEM_LIMIT: u8 = 0xFF;

    pub fn new(calls: &'a PickCalls, net_id_at: usize) -> Self {
        Self { calls, net_id_at }
    }

    /// The creative entry of the item that the block state at `block` makes: the entry of the same item and aux
    /// value, or else the first entry of the item. `None` for a block that makes no item, an item with no entry, or a
    /// stack limit that cannot be read.
    pub fn entry_for(&self, player: usize, block: usize) -> Option<CreativeEntry> {
        let inventory = InventoryAdapter::get()?;
        let layout = inventory.layout()?;
        let (item, aux) = BlockStateAdapter::get()?
            .with_item_instance(block, |_, layout, stack| {
                // SAFETY: the item instance the game just made for the block; `aux` is the offset of its 16-bit aux value.
                let aux = unsafe { ObjectRead::value::<i16>(stack, layout.aux) };
                (InventoryAdapter::item_of(stack, layout), aux)
            })
            .ok()?;
        let aux = aux?;
        if item == 0 {
            return None;
        }
        // SAFETY: the live local player, on the game thread.
        let registry = unsafe { CreativeRegistry::of_player(player, self.calls) }?;
        let mut first_of_item = None;
        let mut exact = None;
        for entry in registry.entries(CreativeItemEntryLayout::SIZE) {
            let instance = CreativeItemEntryLayout::item_instance(entry);
            if InventoryAdapter::item_of(instance, layout) != item {
                continue;
            }
            // SAFETY: an item instance of a live entry; `aux` is the offset of its 16-bit aux value.
            let entry_aux = unsafe { ObjectRead::value::<i16>(instance, layout.aux) };
            if entry_aux == Some(aux) {
                exact = Some(entry);
                break;
            }
            first_of_item.get_or_insert(entry);
        }
        let entry = exact.or(first_of_item)?;
        // SAFETY: a live entry of the registry held above.
        let creative_net_id = unsafe { CreativeItemEntryLayout::net_id(entry) }?;
        let count = registry.max_stack_size(CreativeItemEntryLayout::item_instance(entry));
        (count != 0 && count != Self::NO_ITEM_LIMIT).then_some(CreativeEntry { creative_net_id, count })
    }

    /// Shows `count` items of the creative item `creative_net_id` in hotbar slot `slot` of `player`, through the game's
    /// own `setItem`. False when the item has no entry any more or a call did not run.
    pub fn predict(&self, player: usize, slot: u8, creative_net_id: u32, count: u8) -> bool {
        let Some(inventory) = InventoryAdapter::get() else { return false };
        // SAFETY: the live local player, on the game thread.
        let Some(registry) = (unsafe { CreativeRegistry::of_player(player, self.calls) }) else { return false };
        let Some(layout) = inventory.layout() else { return false };
        let found = registry.entries(CreativeItemEntryLayout::SIZE).into_iter().find(|entry| {
            // SAFETY: a live entry of the registry held above.
            let net_id = unsafe { CreativeItemEntryLayout::net_id(*entry) };
            net_id == Some(creative_net_id)
        });
        let Some(entry) = found else { return false };
        let instance = CreativeItemEntryLayout::item_instance(entry);
        let item = InventoryAdapter::item_of(instance, layout);
        // SAFETY: an item instance of a live entry; `aux` is the offset of its 16-bit aux value.
        let Some(aux) = (unsafe { ObjectRead::value::<i16>(instance, layout.aux) }) else { return false };
        if item == 0 {
            return false;
        }
        // The stack the pick replaces is kept as a copy, for the undo.
        let replaced = inventory.held_in_slot(player, slot).map(|held| {
            // SAFETY: a stack of the live item vector, the calls resolved for the running build, on the game thread.
            unsafe { OwnedItemStack::copy_of(held, self.calls) }
        });
        // SAFETY: the calls are resolved for the running build, on the game thread.
        let mut stack = unsafe { OwnedItemStack::new(self.calls) };
        // SAFETY: a live item of a creative entry, and the calls the stack was made with.
        if !unsafe { stack.reinit(item, i32::from(count), i32::from(aux), self.calls) } {
            return false;
        }
        let shown = inventory.set_player_slot(player, self.calls.set_item_slot, slot, stack.address());
        PickUndo::keep(slot, shown.then_some(replaced).flatten());
        shown
    }

    /// Takes the stack a pick showed out of hotbar slot `slot`: the stack it replaced goes back, or the slot is
    /// empty again. Only while the slot still holds the shown stack (an item with no net id).
    pub fn unpredict(&self, player: usize, slot: u8) -> bool {
        let Some(inventory) = InventoryAdapter::get() else { return false };
        if inventory.unnamed_stack(player, slot, self.net_id_at).is_none() {
            return false;
        }
        if let Some(replaced) = PickUndo::take(slot) {
            return inventory.set_player_slot(player, self.calls.set_item_slot, slot, replaced.address());
        }
        // SAFETY: the calls are resolved for the running build, on the game thread.
        let empty = unsafe { OwnedItemStack::new(self.calls) };
        inventory.set_player_slot(player, self.calls.set_item_slot, slot, empty.address())
    }

    /// Gives the stack in hotbar slot `slot` the net id the server named for it, only while it has none.
    pub fn adopt(&self, player: usize, slot: u8, net_id: i32) -> bool {
        PickUndo::forget();
        InventoryAdapter::get().is_some_and(|inventory| inventory.name_stack(player, slot, self.net_id_at, net_id))
    }
}
