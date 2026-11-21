/// A view of the pixels of one image of a skin, lent by a `network` read.
///
/// `pixels` points into game memory and holds `len` bytes; `len` is `width * height * 4` and the view is valid
/// only while the event that gave the skin runs. `format` is the game's `mce::ImageFormat` value, copied as it is.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AmmlImageView {
    pub width: u32,
    pub height: u32,
    pub format: u32,
    pub _reserved: u32,
    pub pixels: *const u8,
    pub len: usize,
}

impl AmmlImageView {
    /// A view of no pixels.
    pub const EMPTY: Self = Self { width: 0, height: 0, format: 0, _reserved: 0, pixels: core::ptr::null(), len: 0 };
}

const _: () = assert!(core::mem::size_of::<AmmlImageView>() == 32);
