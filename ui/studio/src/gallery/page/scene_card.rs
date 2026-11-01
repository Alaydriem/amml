use amml_ui::gallery::{Sample, SheetGrid};
use egui::Ui;

use crate::component::library::{SceneCard, SceneCardView};
use crate::gallery::GalleryState;
use crate::tokens::selector::SceneColor;

/// Paper: Editor components, `Editor / Scene card` (`G63-0`). One card per scene colour.
pub struct SceneCardPage;

impl SceneCardPage {
    /// Sheet chrome: the card width.
    const CARD: f32 = 316.0;

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        SheetGrid::page(ui, "Scene card", |ui| {
            let cards = [
                ("LIME", SceneColor::Lime, "Arrival", "00:28.80 → 00:37.40"),
                ("PINK", SceneColor::Pink, "Portal run", "00:37.90 → 00:52.00"),
                ("ORCHID", SceneColor::Orchid, "Fortress", "00:12.00 → 00:28.30"),
                ("PERIWINKLE", SceneColor::Periwinkle, "Multi-angle B", "00:31.20 → 00:36.60"),
            ];
            for (caption, color, name, range) in cards {
                let view = SceneCardView { name: Sample::text(name), range: Sample::text(range), color };
                SheetGrid::section(ui, caption, |ui| {
                    ui.scope(|ui| {
                        ui.set_width(Self::CARD);
                        SceneCard::new(&view).show(ui);
                    });
                });
            }
        });
    }
}
