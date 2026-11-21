use crate::Color;

/// What an image is multiplied by: a colour and an alpha. Images of one texture and one tint are drawn together.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tint {
    pub color: Color,
    pub alpha: f32,
}

impl Tint {
    /// No change: white at full alpha.
    pub const WHITE: Self = Self { color: Color::WHITE, alpha: 1.0 };

    pub const fn new(color: Color, alpha: f32) -> Self {
        Self { color, alpha }
    }
}

impl Default for Tint {
    fn default() -> Self {
        Self::WHITE
    }
}
