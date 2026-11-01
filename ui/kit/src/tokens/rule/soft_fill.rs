use egui::Color32;

use crate::tokens::OpacityToken;
use crate::tokens::opacity::Soft;

/// The soft fill rule: a role colour at `--opacity-soft`, behind its own text (chips, pills,
/// badges). Paper cannot mix a token with an opacity, so the canvas writes these fills as `rgb()`
/// at 14 %; code derives them from the role token here.
pub struct SoftFill;

impl SoftFill {
    /// `color` with its RGB kept and its alpha set to `round(255 * Soft::VALUE)`.
    pub fn of(color: Color32) -> Color32 {
        let [r, g, b, _] = color.to_srgba_unmultiplied();
        Color32::from_rgba_unmultiplied(r, g, b, Self::alpha())
    }

    fn alpha() -> u8 {
        // Clamped to 0..=255 first, so the cast cannot truncate.
        (255.0 * <Soft as OpacityToken>::VALUE).round().clamp(0.0, 255.0) as u8
    }
}
