use amml_i18n::text::{Text, UserText};
use amml_ui::gallery::SheetGrid;
use egui::{Ui, vec2};

use crate::component::bar::{SceneSwitcherChip, SceneSwitcherChipView};
use crate::gallery::GalleryState;
use crate::tokens::selector::SceneColor;

/// Paper: Editor components, `Scene switcher chip (closed)` (`84X-0`). The chip at rest, open, for the Original
/// replay, and for a scene with another colour.
pub struct SceneSwitcherChipPage;

impl SceneSwitcherChipPage {
    /// Sheet chrome: the caption column.
    const CAPTION: f32 = 160.0;
    /// Sheet chrome: the row that holds one chip.
    const ROW: egui::Vec2 = vec2(400.0, 30.0);

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        SheetGrid::page(ui, "Scene switcher chip (closed)", |ui| {
            let rows = [
                ("DEFAULT", Some(("Arrival", SceneColor::Lime)), Some((1, 5)), false),
                ("OPEN (MENU SHOWN)", Some(("Arrival", SceneColor::Lime)), Some((1, 5)), true),
                ("ORIGINAL", None, None, false),
                ("MULTI-ANGLE B", Some(("Multi-angle B", SceneColor::Periwinkle)), Some((4, 5)), false),
            ];
            for (caption, scene, position, open) in rows {
                let view = SceneSwitcherChipView {
                    scene: scene.map(|(name, colour)| (Text::user(UserText::from_model(name.to_owned())), colour)),
                    position,
                    open,
                };
                SheetGrid::labelled(ui, caption, Self::CAPTION, SheetGrid::COLUMN_GAP, Self::ROW, |ui, _| {
                    SceneSwitcherChip::new(&view).show(ui);
                });
            }
        });
    }
}
