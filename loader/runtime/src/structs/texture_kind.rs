use crate::TextureTarget;

/// What a texture of the book is made of.
pub(crate) enum TextureKind {
    File { target: TextureTarget },
    Upload { name: String, width: u32, height: u32, pixels: Vec<u8>, uploaded: bool },
}
