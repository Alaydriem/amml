use amml_abi::AmmlTextureReload;

/// Why the loader bound or freed the mod textures (`TexturesReloaded`). A value this crate does not know is `Other`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextureReload {
    /// The active resource pack stack changed. Every `TextureId` stays valid: the loader binds each file texture again
    /// and uploads each pixel texture again from its own copy. A mod that draws pixels it makes each frame may upload
    /// them again.
    ResourcePack,
    /// The render device was reset. Every `TextureId` is stale: load or upload each texture again.
    DeviceReset,
    Other,
}

impl From<AmmlTextureReload> for TextureReload {
    fn from(reason: AmmlTextureReload) -> Self {
        match reason {
            AmmlTextureReload::RESOURCE_PACK => Self::ResourcePack,
            AmmlTextureReload::DEVICE_RESET => Self::DeviceReset,
            _ => Self::Other,
        }
    }
}
