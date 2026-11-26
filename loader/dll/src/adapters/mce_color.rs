use amml_abi::AmmlRgba;

/// The game's `mce::Color`: four floats from 0 to 1. A member function gives it through a hidden pointer.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
#[repr(C)]
pub struct MceColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl MceColor {
    /// The colour times `tint`, channel by channel; the alpha stays this colour's.
    pub fn tinted(self, tint: MceColor) -> Self {
        Self { r: self.r * tint.r, g: self.g * tint.g, b: self.b * tint.b, a: self.a }
    }

    /// The colour as bytes: each channel rounded to the nearest of 0 to 255, alpha 255.
    pub fn to_rgba(self) -> AmmlRgba {
        AmmlRgba { r: Self::channel(self.r), g: Self::channel(self.g), b: Self::channel(self.b), a: 255 }
    }

    // A channel is clamped to 0 to 1; a value that is not a number gives 0.
    fn channel(value: f32) -> u8 {
        (value.clamp(0.0, 1.0) * 255.0 + 0.5) as u8
    }
}

const _: () = assert!(size_of::<MceColor>() == 16);
