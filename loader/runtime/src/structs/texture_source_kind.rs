/// Where a texture file is: the folder of a mod, or the game and its resource packs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextureSourceKind {
    ModFile,
    Vanilla,
}
