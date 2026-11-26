use amml_abi::AmmlScreenKind;

/// The kind of a screen the game draws. A screen this crate does not tell apart is `Other`.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ScreenKind {
    SmallChest,
    LargeChest,
    Barrel,
    ShulkerBox,
    EnderChest,
    Hopper,
    Dispenser,
    Dropper,
    PlayerInventory,
    #[default]
    Other,
}

impl From<AmmlScreenKind> for ScreenKind {
    fn from(kind: AmmlScreenKind) -> Self {
        match kind {
            AmmlScreenKind::SMALL_CHEST => Self::SmallChest,
            AmmlScreenKind::LARGE_CHEST => Self::LargeChest,
            AmmlScreenKind::BARREL => Self::Barrel,
            AmmlScreenKind::SHULKER_BOX => Self::ShulkerBox,
            AmmlScreenKind::ENDER_CHEST => Self::EnderChest,
            AmmlScreenKind::HOPPER => Self::Hopper,
            AmmlScreenKind::DISPENSER => Self::Dispenser,
            AmmlScreenKind::DROPPER => Self::Dropper,
            AmmlScreenKind::PLAYER_INVENTORY => Self::PlayerInventory,
            _ => Self::Other,
        }
    }
}

impl From<ScreenKind> for AmmlScreenKind {
    fn from(kind: ScreenKind) -> Self {
        match kind {
            ScreenKind::SmallChest => Self::SMALL_CHEST,
            ScreenKind::LargeChest => Self::LARGE_CHEST,
            ScreenKind::Barrel => Self::BARREL,
            ScreenKind::ShulkerBox => Self::SHULKER_BOX,
            ScreenKind::EnderChest => Self::ENDER_CHEST,
            ScreenKind::Hopper => Self::HOPPER,
            ScreenKind::Dispenser => Self::DISPENSER,
            ScreenKind::Dropper => Self::DROPPER,
            ScreenKind::PlayerInventory => Self::PLAYER_INVENTORY,
            ScreenKind::Other => Self::OTHER,
        }
    }
}
