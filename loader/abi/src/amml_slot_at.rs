/// One slot of a container screen: a collection of the screen and an index in it.
///
/// `collection` is one of the constants below; any other value is read as `OTHER`. `index` is the position in the
/// collection: a container slot, a hotbar slot 0 to 8, or an inventory slot 0 to 26 (player slot minus 9).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct AmmlSlotAt {
    pub collection: u32,
    pub index: i32,
}

impl AmmlSlotAt {
    /// The slots of the container the screen shows (`container_items`).
    pub const CONTAINER: u32 = 0;
    /// The nine hotbar slots (`hotbar_items`).
    pub const HOTBAR: u32 = 1;
    /// The 27 main inventory slots (`inventory_items`).
    pub const INVENTORY: u32 = 2;
    /// Any collection of the game this API does not name.
    pub const OTHER: u32 = 3;
    /// The largest index a call takes; a larger one is `INVALID_ARGUMENT`. The loader checks the index of a known
    /// collection against the slot count of the screen too.
    pub const MAX_INDEX: i32 = 255;
}

const _: () = assert!(core::mem::size_of::<AmmlSlotAt>() == 8);
