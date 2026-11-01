use amml_i18n::text::{Text, UserText};
use amml_ui::widget::Tooltip;
use egui::{Rect, Ui, pos2};

use super::BarRows;
use crate::component::bar::{SceneSwitcherChipView, StudioMode, TopBar, TopBarView};
use crate::gallery::GalleryState;
use crate::tokens::selector::SceneColor;

/// Paper: Editor components, `Top bar` (`G-0`). Edit, Preview, Read only, and the Original with Export off and its
/// tooltip at 1920, and Edit at 1280.
pub struct TopBarPage;

impl TopBarPage {
    /// Paper: the bar artboards are 1920 wide; the narrow row is the 1280 window.
    pub const WIDTH: f32 = 1920.0;
    const SMALL: f32 = 1280.0;
    // Paper `K4H-0`: the tooltip is 10 under the bar, its right edge at the bar's 12 right padding
    const TOOLTIP_GAP: f32 = 10.0;
    const TOOLTIP_RIGHT: f32 = 12.0;
    const BLOCKED: &str = "The Original is the replay as recorded. Pick a scene to export.";

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        BarRows::page(ui, |ui| {
            let rows = [
                ("NORMAL", StudioMode::Edit, true, Self::WIDTH),
                ("PREVIEW", StudioMode::Preview, true, Self::WIDTH),
                ("READ ONLY", StudioMode::Edit, false, Self::WIDTH),
                ("1280", StudioMode::Edit, true, Self::SMALL),
            ];
            for (caption, mode, editable, width) in rows {
                let view = Self::view(mode, editable);
                BarRows::row(ui, caption, width, |ui| {
                    TopBar::new(&view).show(ui);
                });
                if caption == "READ ONLY" {
                    BarRows::row(ui, "ORIGINAL", Self::WIDTH, Self::original);
                }
            }
        });
    }

    // the Original: no scene in the chip, Export muted, and its tooltip drawn open under the bar
    fn original(ui: &mut Ui) {
        let mut view = Self::view(StudioMode::Edit, true);
        view.scene.scene = None;
        view.scene.position = None;
        view.export_blocked = Some(Self::words(Self::BLOCKED));
        TopBar::new(&view).show(ui);
        let tooltip = Tooltip::new(Self::words(Self::BLOCKED));
        // measured off-screen first: the bubble is placed by its right edge
        let size = tooltip.paint(&ui.painter().with_clip_rect(Rect::NOTHING), pos2(0.0, 0.0)).size();
        let top = ui.min_rect().bottom() + Self::TOOLTIP_GAP;
        let right = ui.max_rect().right() - Self::TOOLTIP_RIGHT;
        tooltip.paint(ui.painter(), pos2(right - size.x, top));
    }

    fn words(text: &str) -> Text {
        Text::user(UserText::from_model(text.to_owned()))
    }

    fn view(mode: StudioMode, editable: bool) -> TopBarView {
        TopBarView {
            title: Self::words("Nether trip"),
            mode,
            can_undo: editable,
            can_redo: false,
            history_open: false,
            scene: SceneSwitcherChipView {
                scene: Some((Self::words("Arrival"), SceneColor::Lime)),
                position: Some((1, 5)),
                open: false,
            },
            export_blocked: None,
        }
    }
}
