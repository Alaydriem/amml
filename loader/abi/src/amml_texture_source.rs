/// Where `texture_load` finds a texture.
///
/// A plain integer so that a value from the other side of the boundary can never be invalid; a loader refuses
/// a value it does not know.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AmmlTextureSource(pub u8);

impl AmmlTextureSource {
    /// A file in the folder of the mod. The path is relative to it, with or without the `.png` extension.
    pub const MOD_FILE: Self = Self(1);
    /// A texture of the game or of its resource packs, by its path without extension.
    pub const VANILLA: Self = Self(2);
}
