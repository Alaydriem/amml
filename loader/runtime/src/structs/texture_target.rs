/// What the game is asked for when a texture is bound.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TextureTarget {
    /// A texture of the game or its resource packs, by the path without extension (file system 0).
    Vanilla(String),
    /// A file by its absolute path without extension (the raw file system).
    Raw(String),
    /// A texture the loader uploaded, by its name in the client texture group.
    Group(String),
}
