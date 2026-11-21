use amml_abi::AmmlTextureId;

/// A texture of the loader. `id` is never given twice in a process. `generation` counts the times the loader freed
/// every texture (a world leave, a render device reset): a `TextureId` of an older generation gives `Stale`. Free
/// it with `Textures::unload`; the loader also frees it when its mod is disabled.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TextureId {
    pub id: u64,
    pub generation: u32,
}

impl From<AmmlTextureId> for TextureId {
    fn from(texture: AmmlTextureId) -> Self {
        Self { id: texture.id, generation: texture.generation }
    }
}

impl From<TextureId> for AmmlTextureId {
    fn from(texture: TextureId) -> Self {
        Self { id: texture.id, generation: texture.generation, _reserved: 0 }
    }
}
