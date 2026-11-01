use egui::{Rect, Ui, pos2, vec2};

use crate::gallery::{GalleryState, Sample, SheetGrid};
use crate::tokens::{SpacingToken, spacing};
use crate::widget::{ValueRow, ValueRowSize};

/// Paper: Widgets, `Widget / Value row` (`GCL-0`).
pub struct ValueRowSheet;

impl ValueRowSheet {
    /// Sheet chrome: the width of one row, and the gap between the compact rows.
    const WIDTH: f32 = 360.0;
    const COMPACT_GAP: f32 = 4.0;

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        let large = <spacing::Row as SpacingToken>::VALUE;
        let compact = <spacing::ControlSmall as SpacingToken>::VALUE;
        SheetGrid::page(ui, "Value row", |ui| {
            SheetGrid::panel(ui, |ui| {
                SheetGrid::labelled(
                    ui,
                    "Large",
                    SheetGrid::LABEL_WIDTH,
                    SheetGrid::COLUMN_GAP,
                    vec2(Self::WIDTH, large),
                    |ui, rect| {
                        ValueRow::new(Sample::text("Length"), Sample::text("02:14:08")).paint(ui.painter(), rect);
                    },
                );
                SheetGrid::labelled(
                    ui,
                    "Large, long value",
                    SheetGrid::LABEL_WIDTH,
                    SheetGrid::COLUMN_GAP,
                    vec2(Self::WIDTH, large),
                    |ui, rect| {
                        ValueRow::new(Sample::text("Voice"), Sample::text("BVC session linked"))
                            .words()
                            .paint(ui.painter(), rect);
                    },
                );
                let rows = [("Length", "8.60 s, 172 ticks"), ("Output", "6.20 s"), ("Timelapse", "ends follow range")];
                let height = compact * rows.len() as f32 + Self::COMPACT_GAP * (rows.len() as f32 - 1.0);
                SheetGrid::labelled(
                    ui,
                    "Compact",
                    SheetGrid::LABEL_WIDTH,
                    SheetGrid::COLUMN_GAP,
                    vec2(Self::WIDTH, height),
                    |ui, rect| {
                        for (index, (label, value)) in rows.into_iter().enumerate() {
                            let top = rect.top() + (compact + Self::COMPACT_GAP) * index as f32;
                            let row = Rect::from_min_size(pos2(rect.left(), top), vec2(Self::WIDTH, compact));
                            ValueRow::new(Sample::text(label), Sample::text(value))
                                .size(ValueRowSize::Compact)
                                .paint(ui.painter(), row);
                        }
                    },
                );
            });
        });
    }
}
