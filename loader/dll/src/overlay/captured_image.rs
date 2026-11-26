/// The pixels of one back-buffer copy, read back from its staging texture: `height` rows of `width * 4` bytes with no
/// padding, in the back buffer's format (`format`, a `DXGI_FORMAT` value that `PixelConverter::supports`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapturedImage {
    pub width: u32,
    pub height: u32,
    pub format: u32,
    pub pixels: Vec<u8>,
}
