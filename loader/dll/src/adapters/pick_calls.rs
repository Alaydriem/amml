use crate::LoaderState;

/// The derived values the creative pick needs besides the request calls: the level of the player, the creative item
/// list behind it, the stack constructor and `reinit`, the slot write and the stack limit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PickCalls {
    pub level_at: usize,
    pub item_registry_slot: usize,
    pub creative_registry: usize,
    pub creative_items_at: usize,
    pub max_stack: usize,
    pub empty_ctor: usize,
    pub copy_ctor: usize,
    pub stack_vtable: usize,
    pub reinit_slot: usize,
    pub set_item_slot: usize,
}

impl PickCalls {
    /// The calls from the resolved entries; `None` when one is missing.
    pub fn new(state: &LoaderState) -> Option<Self> {
        let value = |name: &str| state.resolve(name).ok().and_then(|value| usize::try_from(value).ok());
        Some(Self {
            // An offset of 0 would read the vtable pointer of the player as its level.
            level_at: value("Actor.mLevel").filter(|value| *value != 0)?,
            item_registry_slot: value("Level.getItemRegistry")?,
            creative_registry: value("ItemRegistryRef.getCreativeItemRegistry").filter(|value| *value != 0)?,
            creative_items_at: value("CreativeItemRegistry.mCreativeItems")?,
            max_stack: value("ItemStackBase.getMaxStackSize").filter(|value| *value != 0)?,
            empty_ctor: value("ItemStack.emptyCtor").filter(|value| *value != 0)?,
            copy_ctor: value("ItemStack.copyCtor").filter(|value| *value != 0)?,
            stack_vtable: value("ItemStack.vtable").filter(|value| *value != 0)?,
            reinit_slot: value("ItemStack.reinit")?,
            set_item_slot: value("FillingContainer.setItem")?,
        })
    }
}
