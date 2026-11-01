use std::path::Path;

use amml_ui::tokens::{ColorToken, OpacityToken, SpacingToken, color, opacity, spacing};
use amml_ui_gen::app::Generator;
use amml_ui_gen::model::{OwnershipFile, TokenFile};

#[test]
fn accent_matches_paper() {
    assert_eq!(<color::Accent as ColorToken>::VALUE, egui::Color32::from_rgb(0x8B, 0x7C, 0xFF));
    assert_eq!(<color::Accent as ColorToken>::NAME, "--color-accent");
}

#[test]
fn focus_ring_is_accent_hover() {
    assert_eq!(<color::FocusRing as ColorToken>::VALUE, <color::AccentHover as ColorToken>::VALUE);
}

#[test]
fn row_is_28() {
    assert_eq!(<spacing::Row as SpacingToken>::VALUE, 28.0);
}

#[test]
fn soft_is_14_percent() {
    assert_eq!(<opacity::Soft as OpacityToken>::VALUE, 0.14);
}

#[test]
fn axis_and_playhead_colours_are_amml_ui_tokens() {
    use amml_ui::tokens::selector::AxisColor;
    let axes =
        [<color::AxisX as ColorToken>::VALUE, <color::AxisY as ColorToken>::VALUE, <color::AxisZ as ColorToken>::VALUE];
    assert_eq!(AxisColor::ALL.map(AxisColor::color), axes);
    assert_eq!(<color::Playhead as ColorToken>::VALUE, egui::Color32::from_rgb(0xFF, 0x4D, 0x5E));
}

#[test]
fn generation_is_idempotent() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let read = |name: &str| {
        let path = root.join("design").join(name);
        std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
    };
    let tokens: TokenFile = serde_json::from_str(&read("tokens.json")).unwrap();
    let owners: OwnershipFile = serde_json::from_str(&read("ownership.json")).unwrap();
    let (kit, studio) = Generator::build(&tokens, &owners).unwrap();
    assert_eq!(kit.diff(&root.join("src/tokens/generated")).unwrap(), Vec::<std::path::PathBuf>::new());
    assert_eq!(studio.diff(&root.join("../studio/src/tokens/generated")).unwrap(), Vec::<std::path::PathBuf>::new());
}
