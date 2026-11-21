use amml_abi::AmmlRgba;

/// A colour of four 8-bit channels.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl From<AmmlRgba> for Rgba {
    fn from(color: AmmlRgba) -> Self {
        Self { r: color.r, g: color.g, b: color.b, a: color.a }
    }
}

impl From<Rgba> for AmmlRgba {
    fn from(color: Rgba) -> Self {
        Self { r: color.r, g: color.g, b: color.b, a: color.a }
    }
}
