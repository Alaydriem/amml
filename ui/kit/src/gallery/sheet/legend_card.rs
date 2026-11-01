use egui::{Ui, vec2};

use crate::gallery::{GalleryState, Sample, SheetGrid};
use crate::tokens::{ColorToken, color};
use crate::widget::{LegendCard, LegendRow};

/// Paper: Widgets, `Widget / Legend card` (`GCN-0`). The camera colours are amml-studio-ui tokens, so the page shows
/// three amml-ui colours in their place.
pub struct LegendCardSheet;

impl LegendCardSheet {
    /// Sheet chrome: the card width.
    const WIDTH: f32 = 476.0;

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        SheetGrid::page(ui, "Legend card", |ui| {
            SheetGrid::panel(ui, |ui| {
                let card = LegendCard::new(Sample::text("In-world camera path  ·  Arrival"))
                    .note(Sample::text("Only the active scene's path."))
                    .rows(vec![
                        LegendRow::new(
                            <color::Warning as ColorToken>::VALUE,
                            Sample::text("Key frustum, triangle = up (roll)"),
                        ),
                        LegendRow::new(<color::OnAccent as ColorToken>::VALUE, Sample::text("Selected key")),
                        LegendRow::new(
                            <color::Danger as ColorToken>::VALUE,
                            Sample::text("Scene camera at the playhead"),
                        ),
                    ]);
                let size = vec2(Self::WIDTH, card.height());
                SheetGrid::labelled(
                    ui,
                    "Three rows",
                    SheetGrid::LABEL_WIDTH,
                    SheetGrid::COLUMN_GAP,
                    size,
                    |ui, rect| {
                        card.paint(ui.painter(), rect.min, Self::WIDTH);
                    },
                );
            });
        });
    }
}
