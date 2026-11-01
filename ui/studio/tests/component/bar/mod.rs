mod scene_switcher_chip;
mod scene_switcher_list;
mod status_bar;
mod top_bar;
mod transport;

use amml_studio_ui::component::bar::SceneSwitcherChipView;
use amml_studio_ui::tokens::selector::SceneColor;

use crate::glyph::words;

// the chip for scene 1 of 5, "Arrival"
pub fn arrival() -> SceneSwitcherChipView {
    SceneSwitcherChipView { scene: Some((words("Arrival"), SceneColor::Lime)), position: Some((1, 5)), open: false }
}
