/// Why the texture book refuses a call.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum TextureRefusal {
    #[error("the texture is of an older generation, or was freed")]
    Stale,
    #[error("the texture belongs to another mod")]
    NotOwner,
    #[error("the path is empty")]
    EmptyPath,
    #[error("the path leaves the mod folder")]
    OutsideModFolder,
    #[error("the size is zero or the pixels do not fit it")]
    BadImage,
}
