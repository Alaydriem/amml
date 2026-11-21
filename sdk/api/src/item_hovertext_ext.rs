use amml_abi::ItemHovertext;

use crate::HovertextKind;

/// Typed reads and the one write of an `ItemHovertext`.
pub trait ItemHovertextExt {
    /// What item the text is for.
    fn kind(&self) -> HovertextKind;

    /// Leaves the contents lines of the item out of the text; the name and enchantment lines stay.
    fn hide_contents(&mut self);
}

impl ItemHovertextExt for ItemHovertext {
    fn kind(&self) -> HovertextKind {
        HovertextKind::from(self.kind)
    }

    fn hide_contents(&mut self) {
        self.contents_lines = 0;
    }
}
