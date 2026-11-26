use crate::TextureKey;
use crate::TextureTarget;

/// One game call the texture pump makes for the book.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TextureJob {
    /// Asks the game for `target`, then reports the answer with `TextureBook::bound`.
    Bind { key: TextureKey, target: TextureTarget },
    /// Uploads the pixels under `name` into the client texture group, reports it with `TextureBook::uploaded`,
    /// then asks the group for `name` and reports the answer with `TextureBook::bound`.
    Upload { key: TextureKey, name: String, width: u32, height: u32, pixels: Vec<u8> },
}
