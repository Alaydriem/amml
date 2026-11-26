use std::ops::BitOr;

use amml_abi::AmmlItemDrawFlags;

/// How `UiDraw::item` draws an item: a bit set. `ItemDrawFlags::NONE` is the plain item; `FOIL_ONLY` also draws the
/// enchantment glint.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ItemDrawFlags(u32);

impl ItemDrawFlags {
    /// No flag.
    pub const NONE: Self = Self(AmmlItemDrawFlags::NONE.0);
    /// Draw the enchantment glint on the item.
    pub const FOIL_ONLY: Self = Self(AmmlItemDrawFlags::FOIL_ONLY.0);

    /// Whether every flag of `other` is set.
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

impl BitOr for ItemDrawFlags {
    type Output = Self;

    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl From<ItemDrawFlags> for AmmlItemDrawFlags {
    fn from(flags: ItemDrawFlags) -> Self {
        Self(flags.0)
    }
}
