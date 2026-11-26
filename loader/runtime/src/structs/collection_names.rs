use amml_abi::{AmmlScreenKind, AmmlSlotAt};

/// The names of the game's item collections of a container screen, and the slot counts of the screens the API
/// tells apart. A mod never sees a game name: it uses the collection constants of `AmmlSlotAt`.
pub struct CollectionNames;

impl CollectionNames {
    /// The collection of the stack on the cursor; its one stack is at index 0. It has no `AmmlSlotAt` collection:
    /// the cursor has its own call.
    pub const CURSOR: &'static str = "cursor_items";

    const CONTAINER: &'static str = "container_items";
    const HOTBAR: &'static str = "hotbar_items";
    const INVENTORY: &'static str = "inventory_items";
    const HOTBAR_SLOTS: u32 = 9;
    // The inventory collection holds player slots 9 to 35: its index is the player slot minus 9.
    const INVENTORY_SLOTS: u32 = 27;

    /// The collection of a game collection name; a name the API does not name is `OTHER`.
    pub fn collection_of(name: &str) -> u32 {
        match name {
            Self::CONTAINER => AmmlSlotAt::CONTAINER,
            Self::HOTBAR => AmmlSlotAt::HOTBAR,
            Self::INVENTORY => AmmlSlotAt::INVENTORY,
            _ => AmmlSlotAt::OTHER,
        }
    }

    /// The game name of `collection`; `None` for `OTHER` and for a value that is no collection.
    pub fn game_name(collection: u32) -> Option<&'static str> {
        match collection {
            AmmlSlotAt::CONTAINER => Some(Self::CONTAINER),
            AmmlSlotAt::HOTBAR => Some(Self::HOTBAR),
            AmmlSlotAt::INVENTORY => Some(Self::INVENTORY),
            _ => None,
        }
    }

    /// The number of slots of `collection` on a screen of `kind`: 27, 54, 5, 9 or 0 (the player inventory) for the
    /// container, 9 for the hotbar and 27 for the inventory. `None` for `OTHER`, and for the container of a screen
    /// kind that is not known.
    pub fn slot_count(kind: AmmlScreenKind, collection: u32) -> Option<u32> {
        match collection {
            AmmlSlotAt::CONTAINER => Self::container_slots(kind),
            AmmlSlotAt::HOTBAR => Some(Self::HOTBAR_SLOTS),
            AmmlSlotAt::INVENTORY => Some(Self::INVENTORY_SLOTS),
            _ => None,
        }
    }

    fn container_slots(kind: AmmlScreenKind) -> Option<u32> {
        match kind {
            AmmlScreenKind::SMALL_CHEST
            | AmmlScreenKind::BARREL
            | AmmlScreenKind::SHULKER_BOX
            | AmmlScreenKind::ENDER_CHEST => Some(27),
            AmmlScreenKind::LARGE_CHEST => Some(54),
            AmmlScreenKind::HOPPER => Some(5),
            AmmlScreenKind::DISPENSER | AmmlScreenKind::DROPPER => Some(9),
            AmmlScreenKind::PLAYER_INVENTORY => Some(0),
            _ => None,
        }
    }
}
