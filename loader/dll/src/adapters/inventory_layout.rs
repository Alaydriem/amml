use crate::NameReader;

/// The derived values `InventoryAdapter` reads the player's inventory with: the offsets of the inventory chain, the
/// `ItemStack` mirror (98h bytes, `mItem` at 8, `mCount` at 22h, `mAuxValue` at 20h, `mUserData` at 10h), the two functions and the name reader.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InventoryLayout {
    pub player_inventory: usize,
    pub container: usize,
    pub selected: usize,
    pub items: usize,
    pub stack_size: usize,
    pub item: usize,
    pub count: usize,
    pub aux: usize,
    pub user_data: usize,
    pub damage_fn: usize,
    pub max_damage_slot: usize,
    pub full_name: usize,
    pub names: NameReader,
}

impl InventoryLayout {
    /// The `ItemStack` mirror: its size, the `mItem` pointer and the `mCount` byte.
    pub const STACK_SIZE: u64 = 0x98;
    pub const ITEM_OFFSET: u64 = 0x08;
    pub const COUNT_OFFSET: u64 = 0x22;
    /// `mAuxValue`, a 16-bit value, and `mUserData`, a `unique_ptr<CompoundTag>`: read from the copy constructor
    /// and the destructor of `ItemStackBase` on both gate builds, so they are mirror members, not entries.
    pub const AUX_OFFSET: u64 = 0x20;
    pub const USER_DATA_OFFSET: u64 = 0x10;
    /// The hotbar slots.
    pub const HOTBAR: usize = 9;
    /// The slots of the player inventory: the hotbar, then the main inventory.
    pub const SLOTS: usize = 36;

    /// The layout from the derived entry values `value` gives (`None` for an entry that did not resolve); the
    /// reason when an entry is missing, the `ItemStack` values are not the mirror's, or no name mirror fits.
    pub fn from_entries(value: &impl Fn(&str) -> Option<u64>) -> Result<Self, &'static str> {
        let get = |name: &str| value(name).ok_or("an inventory entry did not resolve");
        let size = |name: &str| get(name).and_then(|value| usize::try_from(value).map_err(|_| "a value is too large"));
        if get("ItemStack.size")? != Self::STACK_SIZE
            || get("ItemStackBase.mItem")? != Self::ITEM_OFFSET
            || get("ItemStackBase.mCount")? != Self::COUNT_OFFSET
        {
            return Err("the ItemStack layout is not the loader's mirror");
        }
        let names = NameReader::new(get("HashedString.layout")?).ok_or("no HashedString mirror for this build")?;
        Ok(Self {
            player_inventory: size("Player.playerInventory")?,
            container: size("PlayerInventory.mInventory")?,
            selected: size("PlayerInventory.mSelected")?,
            items: size("FillingContainer.mItems")?,
            stack_size: Self::STACK_SIZE as usize,
            item: Self::ITEM_OFFSET as usize,
            count: Self::COUNT_OFFSET as usize,
            aux: Self::AUX_OFFSET as usize,
            user_data: Self::USER_DATA_OFFSET as usize,
            damage_fn: size("ItemStackBase.getDamageValue")?,
            max_damage_slot: size("Item.getMaxDamage")?,
            full_name: size("Item.mFullName")?,
            names,
        })
    }

    /// The address of the stack in `slot` of an item vector from `first` to `last`; `None` for a null vector, a
    /// vector whose end is before its start, or a slot past the end.
    pub fn stack_at(&self, first: usize, last: usize, slot: usize) -> Option<usize> {
        if first == 0 || last < first {
            return None;
        }
        let count = (last - first) / self.stack_size;
        (slot < count).then(|| first + slot * self.stack_size)
    }
}
