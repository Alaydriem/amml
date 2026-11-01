use egui::{Align, Align2, Layout, Rect, pos2};

/// Paper: Mod manager, `Widget / Table header + Table row`, a column's text alignment. A numeric
/// column is `End`: its head and its cells sit against the column's right edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TableAlign {
    #[default]
    Start,
    End,
}

impl TableAlign {
    /// The anchor point of a line of text in `cell`, and the anchor it is drawn with.
    pub fn anchor(self, cell: Rect) -> (egui::Pos2, Align2) {
        match self {
            Self::Start => (cell.left_center(), Align2::LEFT_CENTER),
            Self::End => (pos2(cell.right(), cell.center().y), Align2::RIGHT_CENTER),
        }
    }

    /// The layout a cell's widgets are placed with.
    pub fn layout(self) -> Layout {
        match self {
            Self::Start => Layout::left_to_right(Align::Center),
            Self::End => Layout::right_to_left(Align::Center),
        }
    }
}
