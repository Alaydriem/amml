use std::mem::offset_of;

use amml_abi::AmmlImageView;

/// Layout mirror of the game's `mce::Image` (0x30 bytes): the format, the size and the pixel blob.
///
/// No entry proves these offsets. They come from the mirror of the old heads mod, which ran on 26.60, and the first image
/// the loader reads is logged with the values it found. `view` is the check: a blob whose size is not
/// `width * height * 4` gives nothing, so a mirror that is wrong shows as no image and not as a read of random memory.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MceImageLayout {
    pub format: u32,      // 0x00
    pub width: u32,       // 0x04
    pub height: u32,      // 0x08
    depth: u32,           // 0x0C
    usage: u8,            // 0x10
    alpha_usage: u8,      // 0x11
    _reserved: [u8; 6],   // 0x12
    blob_deleter: usize,  // 0x18
    pub data: *const u8,  // 0x20
    pub blob_size: usize, // 0x28
}

impl MceImageLayout {
    /// Largest side of an image a view is made of; a larger value is a misread.
    pub const MAX_SIDE: u32 = 4096;

    /// A view of the pixels; `None` for a zero or too large side, a null blob, or a blob whose size is not
    /// `width * height * 4`.
    pub fn view(&self) -> Option<AmmlImageView> {
        if self.width == 0 || self.height == 0 || self.width > Self::MAX_SIDE || self.height > Self::MAX_SIDE {
            return None;
        }
        let expected = (self.width as usize).checked_mul(self.height as usize)?.checked_mul(4)?;
        if self.data.is_null() || self.blob_size != expected {
            return None;
        }
        Some(AmmlImageView {
            width: self.width,
            height: self.height,
            format: self.format,
            _reserved: 0,
            pixels: self.data,
            len: self.blob_size,
        })
    }
}

const _: () = assert!(offset_of!(MceImageLayout, width) == 0x04);
const _: () = assert!(offset_of!(MceImageLayout, data) == 0x20);
const _: () = assert!(offset_of!(MceImageLayout, blob_size) == 0x28);
const _: () = assert!(size_of::<MceImageLayout>() == 0x30);
