/// Paper: Mod manager, `Widget / Table header + Table row`, the row sizes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TableRowSize {
    /// The keybind row: 68 px least height, 20 px above and below the content.
    #[default]
    Regular,
    /// The list row of the materials table: 38 px, one line of content.
    Compact,
}

impl TableRowSize {
    /// Paper: Table, a row's least height. The compact value is the Table sheet's compact row.
    pub fn min_height(self) -> f32 {
        match self {
            Self::Regular => 68.0,
            Self::Compact => 38.0,
        }
    }

    /// Paper: Table, the top and bottom room around a row's content.
    pub fn pad_y(self) -> f32 {
        match self {
            Self::Regular => 20.0,
            Self::Compact => 0.0,
        }
    }
}
