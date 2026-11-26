use crate::structs::texture_kind::TextureKind;

/// One texture of the book.
pub(crate) struct TextureEntry {
    pub(crate) owner: usize,
    pub(crate) kind: TextureKind,
    /// The address of the held `TexturePtr`; 0 while the game gave none.
    pub(crate) held: usize,
    pub(crate) ready: bool,
    /// The next game call is due at the next pump.
    pub(crate) due: bool,
    /// When the game was asked last, in milliseconds; `None` before the first ask.
    pub(crate) asked_at: Option<u64>,
}
