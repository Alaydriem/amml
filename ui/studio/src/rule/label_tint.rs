use amml_ui::tokens::{ColorToken, color};
use egui::Color32;

/// The label tint rule: text on a track, scene or status fill is that hue moved part of the way to `--color-text`.
/// Paper draws these tints as literals (for example `#F2CE99` on a camera segment); code derives them here, with
/// one amount for every hue.
pub struct LabelTint;

impl LabelTint {
    /// How far the hue moves toward `--color-text`.
    pub const AMOUNT: f32 = 0.5;

    pub fn of(hue: Color32) -> Color32 {
        hue.lerp_to_gamma(<color::Text as ColorToken>::VALUE, Self::AMOUNT)
    }
}
