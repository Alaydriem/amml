use amml_studio_ui::component::bar::{SceneSwitcherChip, SceneSwitcherChipView};

use super::arrival;
use crate::support::UiProbe;

#[test]
fn scene_switcher_chip_with_no_scene() {
    let view = SceneSwitcherChipView { scene: None, position: None, open: false };
    let shapes = UiProbe::new(400.0).shapes(|ui| SceneSwitcherChip::new(&view).show(ui));
    let texts = UiProbe::texts(&shapes);
    assert!(texts.iter().any(|text| text == "Original"), "{texts:?}");
}

#[test]
fn scene_switcher_chip_shows_the_scene_and_its_position() {
    let view = arrival();
    let texts = UiProbe::texts(&UiProbe::new(400.0).shapes(|ui| SceneSwitcherChip::new(&view).show(ui)));
    assert!(texts.iter().any(|text| text == "Arrival"), "{texts:?}");
    assert!(texts.iter().any(|text| text == "1 of 5"), "{texts:?}");
}

#[test]
fn scene_switcher_chip_click_is_reported() {
    let view = arrival();
    let mut probe = UiProbe::new(400.0);
    let shapes = probe.shapes(|ui| SceneSwitcherChip::new(&view).show(ui));
    let at = UiProbe::text_rect(&shapes, "Arrival").unwrap_or_else(|| panic!("no chip")).center();
    assert!(probe.click(at, |ui| SceneSwitcherChip::new(&view).show(ui).clicked()));
}
