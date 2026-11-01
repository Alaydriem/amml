use egui::{Rect, pos2};

use super::TableColumn;

/// Where a table's columns sit in a row: each column's share of the width inside the side insets.
/// The header and the rows use it, so their columns line up.
pub(crate) struct TableLayout;

impl TableLayout {
    /// Paper: Table, the row's left and right inset.
    pub const INSET: f32 = 18.0;

    /// One rect per column, `rect`'s full height.
    pub fn cells(columns: &[TableColumn], rect: Rect) -> Vec<Rect> {
        let inner = rect.shrink2(egui::vec2(Self::INSET, 0.0));
        let mut left = inner.left();
        columns
            .iter()
            .map(|column| {
                let right = left + column.width_fraction * inner.width();
                let cell = Rect::from_min_max(pos2(left, rect.top()), pos2(right, rect.bottom()));
                left = right;
                cell
            })
            .collect()
    }
}
