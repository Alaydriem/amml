/// Mirror of `cg::ImageDescription` (20 bytes), as `ImageBuffer(ImageDescription const&)` reads it.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageDescriptionLayout {
    pub width: u32,
    pub height: u32,
    pub format: u32,
    pub color_space: u8,
    pub image_type: u8,
    pub alpha_usage: u8,
    pub reserved: u8,
    pub depth: u32,
}

const _: () = assert!(size_of::<ImageDescriptionLayout>() == 20);

impl ImageDescriptionLayout {
    /// `mce::TextureFormat::RGBA8`.
    pub const RGBA8: u32 = 0x1C;
    const SRGB: u8 = 1;

    /// An RGBA8 sRGB description of `width` x `height`, as `TextureGroup::addEmptyTexture` builds by default.
    pub fn rgba(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            format: Self::RGBA8,
            color_space: Self::SRGB,
            image_type: 0,
            alpha_usage: 0,
            reserved: 0,
            depth: 1,
        }
    }
}
