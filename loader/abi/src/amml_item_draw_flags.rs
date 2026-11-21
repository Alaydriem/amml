/// The flags of `AmmlUiTable::draw_item`: a bit set.
///
/// A plain integer so that a value from the other side of the boundary can never be invalid.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AmmlItemDrawFlags(pub u32);

impl AmmlItemDrawFlags {
    /// No flag.
    pub const NONE: Self = Self(0);
    /// Draw the enchantment glint on the item.
    pub const FOIL_ONLY: Self = Self(1);

    /// Whether every bit of `other` is set.
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// Whether a bit is set that this version does not know.
    pub const fn has_unknown_bits(self) -> bool {
        self.0 & !Self::FOIL_ONLY.0 != 0
    }
}
