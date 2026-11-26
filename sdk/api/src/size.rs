use amml_abi::AmmlSize;

/// A size in pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl From<AmmlSize> for Size {
    fn from(size: AmmlSize) -> Self {
        Self { width: size.width, height: size.height }
    }
}

impl From<Size> for AmmlSize {
    fn from(size: Size) -> Self {
        Self { width: size.width, height: size.height }
    }
}
