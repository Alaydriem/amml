use amml_studio_ui::tokens::color;
use amml_studio_ui::tokens::selector::{ModeColor, SceneColor, StatusColor, TrackColor};
use amml_ui::tokens::ColorToken;
use egui::Color32;

#[test]
fn scene_lime_is_b5e853() {
    assert_eq!(SceneColor::Lime.color(), Color32::from_rgb(0xB5, 0xE8, 0x53));
}

#[test]
fn selectors_list_every_variant_in_paper_order() {
    assert_eq!(SceneColor::ALL.len(), 6);
    assert_eq!(SceneColor::ALL[5], SceneColor::Sand);
    assert_eq!(TrackColor::ALL.len(), 5);
    assert_eq!(StatusColor::ALL.len(), 5);
    assert_eq!(ModeColor::ALL, [ModeColor::Edit, ModeColor::Preview, ModeColor::ReadOnly, ModeColor::Busy]);
}

#[test]
fn studio_aliases_resolve_to_their_source() {
    assert_eq!(StatusColor::Works.color(), <amml_ui::tokens::color::Success as ColorToken>::VALUE);
    assert_eq!(ModeColor::Preview.color(), TrackColor::Camera.color());
    assert_eq!(<color::SceneCamera as ColorToken>::VALUE, <color::TrackCamera as ColorToken>::VALUE);
}
