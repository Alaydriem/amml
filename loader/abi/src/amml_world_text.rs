/// A text field of the world info that `world_text` reads. A plain integer so that a value from the other side of the
/// boundary can never be invalid.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AmmlWorldText(pub u8);

impl AmmlWorldText {
    /// The folder of the world on disk, as UTF-8.
    pub const STORAGE_FOLDER: Self = Self(1);
    /// The level name of the world.
    pub const LEVEL_NAME: Self = Self(2);
    /// The host of the connection; empty when the game holds none.
    pub const HOST: Self = Self(3);
}
