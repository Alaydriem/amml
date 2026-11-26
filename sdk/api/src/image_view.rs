use crate::PixelFormat;

/// A view of the pixels of one image of a skin. The bytes are the game's own memory, lent for the event that gave the
/// skin: no copy is made, and the view cannot outlive the borrow of the event. `pixels` holds exactly
/// `width * height * 4` bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageView<'f> {
    pub width: u32,
    pub height: u32,
    pub format: PixelFormat,
    pub pixels: &'f [u8],
}
