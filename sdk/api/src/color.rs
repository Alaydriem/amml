use amml_abi::AmmlColor;

/// A colour of three channels, each from 0.0 to 1.0. The alpha is a separate argument of a draw call.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color {
    pub const WHITE: Self = Self { r: 1.0, g: 1.0, b: 1.0 };
    pub const BLACK: Self = Self { r: 0.0, g: 0.0, b: 0.0 };
}

impl From<AmmlColor> for Color {
    fn from(color: AmmlColor) -> Self {
        Self { r: color.r, g: color.g, b: color.b }
    }
}

impl From<Color> for AmmlColor {
    fn from(color: Color) -> Self {
        Self { r: color.r, g: color.g, b: color.b }
    }
}
