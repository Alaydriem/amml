/// Why the loader bound or freed the mod textures (`TexturesReloaded`).
///
/// A plain integer so that a value from the other side of the boundary can never be invalid.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AmmlTextureReload(pub u8);

impl AmmlTextureReload {
    /// The active resource pack stack changed. Every texture id stays valid: the loader binds each file texture again
    /// and uploads each pixel texture again from its own copy.
    pub const RESOURCE_PACK: Self = Self(0);
    /// The render device was reset. Every texture id is stale: a mod loads or uploads its textures again.
    pub const DEVICE_RESET: Self = Self(1);
}
