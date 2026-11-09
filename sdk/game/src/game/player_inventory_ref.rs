use std::ffi::c_void;
use std::ptr::NonNull;

use crate::game::{ItemStackBaseRef, ItemStackLayout, RawObject};
use crate::{EntryId, GameEntries};

/// A player's `PlayerInventory` and the `Inventory` (a `FillingContainer`) it holds: the selected hotbar slot and
/// the stacks of the container's item vector. Read-only. Client main thread only: the game changes the vector on
/// that thread, and nothing here locks it.
pub struct PlayerInventoryRef<'a> {
    supplies: NonNull<c_void>,
    container: NonNull<c_void>,
    items: u64,
    entries: &'a GameEntries,
}

impl<'a> PlayerInventoryRef<'a> {
    // PlayerInventory: two 8-byte listener bases, then mSelected (int); mInventory is a unique_ptr<Inventory>. The
    // Player.playerInventory recipes keep the bytes of both offsets, so the entry fails when they move.
    const SELECTED: u64 = 0x10;
    const INVENTORY: u64 = 0xB8;

    /// The inventory of `player`. `None` for a null player, PlayerInventory or Inventory, or when the
    /// `Player.playerInventory` or `FillingContainer.mItems` entry is off.
    ///
    /// # Safety
    /// `player` must be null or a live `Player` of the game build `entries` was resolved for, and the inventory must
    /// stay live while the wrapper is used.
    #[allow(unsafe_code)]
    pub unsafe fn of_player(player: *mut c_void, entries: &'a GameEntries) -> Option<Self> {
        let player_inventory = entries.get(EntryId::PLAYER_PLAYER_INVENTORY)?;
        let items = entries.get(EntryId::FILLING_CONTAINER_ITEMS)?;
        if player.is_null() {
            return None;
        }
        // SAFETY: the caller guarantees a live player; the entry is the offset of its unique_ptr<PlayerInventory>.
        let supplies = unsafe { RawObject::pointer(player, player_inventory) }?;
        // SAFETY: a live PlayerInventory, whose unique_ptr<Inventory> is at INVENTORY.
        let container = unsafe { RawObject::pointer(supplies, Self::INVENTORY) }?;
        Some(Self { supplies: NonNull::new(supplies)?, container: NonNull::new(container)?, items, entries })
    }

    /// The selected hotbar slot (`PlayerInventory::mSelected`).
    #[allow(unsafe_code)]
    pub fn selected(&self) -> Option<i32> {
        // SAFETY: the live PlayerInventory holds mSelected at SELECTED.
        unsafe { RawObject::read::<i32>(self.supplies.as_ptr(), Self::SELECTED) }
    }

    /// The stack in `slot`. `None` for a slot past the end of the vector or for a null vector.
    #[allow(unsafe_code)]
    pub fn item(&self, slot: usize) -> Option<ItemStackBaseRef<'a>> {
        // SAFETY: the live container holds mItems (first, last, end of storage) at the entry's offset.
        let [first, last] = unsafe { RawObject::read::<[usize; 2]>(self.container.as_ptr(), self.items) }?;
        if first == 0 {
            return None;
        }
        let count = last.checked_sub(first)? / size_of::<ItemStackLayout>();
        if slot >= count {
            return None;
        }
        // SAFETY: the slot is inside the live item vector, so the address is in the same allocation.
        let stack = unsafe { (first as *mut ItemStackLayout).add(slot) };
        Some(ItemStackBaseRef::new(NonNull::new(stack.cast::<c_void>())?, self.entries))
    }

    /// The stack in the selected slot. `None` as [`PlayerInventoryRef::item`], or for a negative selected slot.
    pub fn selected_item(&self) -> Option<ItemStackBaseRef<'a>> {
        let slot = usize::try_from(self.selected()?).ok()?;
        self.item(slot)
    }
}
