use amml_abi::AmmlSlotAt;

/// A group of slots of a container screen. A collection the API does not name is `Other`.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Collection {
    /// The slots of the container the screen shows: 27 for a small chest, 54 for a large one, 5 for a hopper, 9 for a
    /// dispenser, none for the player inventory.
    Container,
    /// The nine hotbar slots.
    Hotbar,
    /// The 27 main inventory slots. The index is the player slot minus 9.
    Inventory,
    Other,
}

impl Collection {
    pub(crate) fn raw(self) -> u32 {
        match self {
            Self::Container => AmmlSlotAt::CONTAINER,
            Self::Hotbar => AmmlSlotAt::HOTBAR,
            Self::Inventory => AmmlSlotAt::INVENTORY,
            Self::Other => AmmlSlotAt::OTHER,
        }
    }
}

impl From<u32> for Collection {
    fn from(raw: u32) -> Self {
        match raw {
            AmmlSlotAt::CONTAINER => Self::Container,
            AmmlSlotAt::HOTBAR => Self::Hotbar,
            AmmlSlotAt::INVENTORY => Self::Inventory,
            _ => Self::Other,
        }
    }
}
