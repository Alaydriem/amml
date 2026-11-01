use amml_ui::tokens::{ColorToken, SpacingToken};

struct Probe;

impl ColorToken for Probe {
    const NAME: &'static str = "--color-probe";
    const VALUE: egui::Color32 = egui::Color32::from_rgb(1, 2, 3);
}

impl SpacingToken for Probe {
    const NAME: &'static str = "--spacing-probe";
    const VALUE: f32 = 8.0;
}

#[test]
fn a_type_can_be_a_color_and_a_spacing_token() {
    assert_eq!(<Probe as ColorToken>::VALUE, egui::Color32::from_rgb(1, 2, 3));
    assert_eq!(<Probe as SpacingToken>::VALUE, 8.0);
    assert_eq!(<Probe as ColorToken>::NAME, "--color-probe");
}
