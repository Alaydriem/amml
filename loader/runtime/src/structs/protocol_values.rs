/// The protocol constants of the item stack request path. They are network protocol numbers, which the game
/// appends to and does not renumber, so they are Rust constants and not derived entries.
///
/// The packet ids come from `inspect packets` on both gate builds, and the game test of the deriver checks them on each gate build; the container ids are the ids the
/// vanilla client sent in recorded requests.
pub struct ProtocolValues;

impl ProtocolValues {
    pub const INTERACT_ID: u32 = 33;
    pub const CONTAINER_OPEN_ID: u32 = 46;
    pub const CONTAINER_CLOSE_ID: u32 = 47;
    pub const ITEM_STACK_REQUEST_ID: u32 = 147;
    pub const ITEM_STACK_RESPONSE_ID: u32 = 148;

    /// The request container id of chests, the ender chest, the hopper, the dispenser and the dropper.
    pub const LEVEL_ENTITY: u8 = 7;
    pub const SHULKER_BOX: u8 = 30;
    pub const BARREL: u8 = 58;
    /// The player's hotbar (slots 0 to 8) and main inventory (slots 9 to 35, sent with the player slot number).
    pub const HOTBAR: u8 = 28;
    pub const INVENTORY: u8 = 29;
    /// The carried stack, slot 0.
    pub const CURSOR: u8 = 59;

    /// `InteractPacketPayload::Action::OpenInventory`.
    pub const OPEN_INVENTORY: u8 = 6;
    /// `SharedTypes::Legacy::ContainerType::None`, the type of the client's close.
    pub const NO_TYPE: i8 = -9;
    /// `SharedTypes::Legacy::ContainerType::Inventory`, the type of the server's open of the player inventory.
    pub const INVENTORY_TYPE: i8 = -1;
    /// `ItemStackNetResult::Success`.
    pub const RESULT_SUCCESS: u8 = 0;

    const HOTBAR_SLOTS: u8 = 9;

    /// The request container id and the slot byte of player slot `slot` (0 to 8 the hotbar, 9 to 35 the main
    /// inventory).
    pub fn player_slot(slot: u8) -> (u8, u8) {
        if slot < Self::HOTBAR_SLOTS { (Self::HOTBAR, slot) } else { (Self::INVENTORY, slot) }
    }
}
