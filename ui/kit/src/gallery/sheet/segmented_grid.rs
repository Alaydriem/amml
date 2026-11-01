use egui::{Id, Ui, vec2};

use crate::gallery::{GalleryState, Sample, SheetGrid};
use crate::state::InteractionState;
use crate::widget::{Segment, Segmented};

/// Paper: Widgets, `Widget / Segmented` (`5MX-0`), the row `Grid, two rows`.
pub struct SegmentedGridSheet;

impl SegmentedGridSheet {
    /// Sheet chrome: the grid width, and its height for two rows (28 high, 5 apart).
    const WIDTH: f32 = 360.0;
    const HEIGHT: f32 = 61.0;

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        SheetGrid::page(ui, "Segmented", |ui| {
            SheetGrid::panel(ui, |ui| {
                let words = ["Sunrise", "Day", "Noon", "Sunset", "Night", "Midnight"];
                let segments = words.iter().map(|word| Segment::Text(Sample::text(word))).collect();
                let mut selected = 2;
                let grid = Segmented::new(Id::NULL, &mut selected, segments).columns(3);
                let size = vec2(Self::WIDTH, Self::HEIGHT);
                SheetGrid::labelled(
                    ui,
                    "Grid, two rows",
                    SheetGrid::LABEL_WIDTH,
                    SheetGrid::COLUMN_GAP,
                    size,
                    |ui, rect| {
                        grid.paint(ui.painter(), rect, 2, InteractionState::Rest);
                    },
                );
            });
        });
    }
}
