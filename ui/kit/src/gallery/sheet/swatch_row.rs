use egui::{Id, Ui, vec2};

use crate::gallery::{GalleryState, SampleColours, SheetGrid};
use crate::widget::SwatchRow;

/// Paper: Widgets, `Widget / Swatch row` (`GCM-0`). The scene colours are amml-studio-ui tokens, so the page shows six
/// amml-ui colours in their place.
pub struct SwatchRowSheet;

impl SwatchRowSheet {
    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        SheetGrid::page(ui, "Swatch row", |ui| {
            SheetGrid::panel(ui, |ui| {
                let mut selected = 0;
                let row = SwatchRow::new(Id::NULL, &mut selected, SampleColours::scene().to_vec());
                let size = vec2(row.width(), SwatchRow::HEIGHT);
                SheetGrid::labelled(
                    ui,
                    "Six colours",
                    SheetGrid::LABEL_WIDTH,
                    SheetGrid::COLUMN_GAP,
                    size,
                    |ui, rect| {
                        row.paint(ui.painter(), rect);
                    },
                );
            });
        });
    }
}
