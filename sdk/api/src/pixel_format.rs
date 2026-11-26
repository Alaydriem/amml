/// The pixel format of an `ImageView`, from the game's `mce::ImageFormat`.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PixelFormat {
    /// Four bytes a pixel: red, green, blue, alpha.
    Rgba8,
    /// A format this crate does not name; the value is the game's.
    Other(u32),
}

impl PixelFormat {
    /// The `mce::ImageFormat` value of RGBA8: 4 on the builds the heads mod ran on. The loader logs the first format it
    /// reads, so a build with another value shows in the log.
    pub const RGBA8: u32 = 4;
}

impl From<u32> for PixelFormat {
    fn from(format: u32) -> Self {
        if format == Self::RGBA8 { Self::Rgba8 } else { Self::Other(format) }
    }
}

impl From<PixelFormat> for u32 {
    fn from(format: PixelFormat) -> Self {
        match format {
            PixelFormat::Rgba8 => PixelFormat::RGBA8,
            PixelFormat::Other(other) => other,
        }
    }
}
