use amml_abi::AmmlScreenKind;

/// The kind of a screen from the name of its root control (the screen name without a namespace).
pub struct ScreenKindTable;

impl ScreenKindTable {
    /// The kind of the screen whose root control is named `name`; a name the table does not know is `OTHER`.
    pub fn kind_of(name: &str) -> AmmlScreenKind {
        match name {
            "small_chest_screen" => AmmlScreenKind::SMALL_CHEST,
            "large_chest_screen" => AmmlScreenKind::LARGE_CHEST,
            "barrel_screen" => AmmlScreenKind::BARREL,
            "shulker_box_screen" => AmmlScreenKind::SHULKER_BOX,
            "ender_chest_screen" => AmmlScreenKind::ENDER_CHEST,
            "hopper_screen" => AmmlScreenKind::HOPPER,
            "dispenser_screen" => AmmlScreenKind::DISPENSER,
            "dropper_screen" => AmmlScreenKind::DROPPER,
            "inventory_screen" | "inventory_screen_pocket" => AmmlScreenKind::PLAYER_INVENTORY,
            _ => AmmlScreenKind::OTHER,
        }
    }
}
