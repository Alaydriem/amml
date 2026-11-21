use amml_abi::{AmmlStatus, AmmlStr, AmmlTextureId, AmmlTextureSource, AmmlUiTable};

use crate::{ApiError, TextureId, TextureSource};

/// The textures of the loader (`Area::UI_TEXTURES`): ask for a file or upload pixels, then draw the texture with
/// `UiDraw::image` once it is ready.
///
/// Every call is allowed from any thread and only queues: the loader binds, uploads and frees at its next UI draw.
/// So a texture is not ready when `load` returns; ask `ready`, or draw it (`UiDraw::image` skips a texture that is
/// not ready). The loader binds every texture again when the game may have dropped its own (a world join, a
/// dimension change) and frees all of them at a world leave and a render device reset: a `TextureId` of before a
/// world leave gives `Stale`, and the mod loads the texture again after the next join. A mod's textures are freed
/// when it is disabled. A mod can free a texture from any thread with `unload`.
#[derive(Clone, Copy)]
pub struct Textures {
    table: Option<AmmlUiTable>,
    module: &'static str,
}

impl Textures {
    pub(crate) fn new(table: Option<AmmlUiTable>, module: &'static str) -> Self {
        Self { table, module }
    }

    /// Asks for the texture `source`. A mod file path that leaves the mod folder gives `Refused`.
    pub fn load(&self, source: TextureSource<'_>) -> Result<TextureId, ApiError> {
        let load = self.table.and_then(|table| table.texture_load).ok_or(ApiError::NotAvailable)?;
        let (kind, path) = match source {
            TextureSource::ModFile(path) => (AmmlTextureSource::MOD_FILE, path),
            TextureSource::Vanilla(path) => (AmmlTextureSource::VANILLA, path),
        };
        let mut out = AmmlTextureId { id: 0, generation: 0, _reserved: 0 };
        // SAFETY: the module id, the path and the output live for the call.
        #[allow(unsafe_code)]
        let status = unsafe { load(AmmlStr::from_str(self.module), kind, AmmlStr::from_str(path), &mut out) };
        ApiError::check(status).map(|()| TextureId::from(out))
    }

    /// Uploads `width` x `height` pixels of four bytes each (red, green, blue, alpha), row by row from the top left.
    /// The pixels are copied before the call returns. A size that is zero, or pixels that do not fit it, give an
    /// error.
    pub fn upload_rgba(&self, width: u32, height: u32, rgba: &[u8]) -> Result<TextureId, ApiError> {
        let upload = self.table.and_then(|table| table.texture_upload_rgba).ok_or(ApiError::NotAvailable)?;
        let mut out = AmmlTextureId { id: 0, generation: 0, _reserved: 0 };
        // SAFETY: the module id, the pixels and the output live for the call.
        #[allow(unsafe_code)]
        let status =
            unsafe { upload(AmmlStr::from_str(self.module), width, height, rgba.as_ptr(), rgba.len(), &mut out) };
        ApiError::check(status).map(|()| TextureId::from(out))
    }

    /// Whether the texture is current and the game gave its data. False for a stale id.
    pub fn ready(&self, texture: TextureId) -> bool {
        let Some(ready) = self.table.and_then(|table| table.texture_ready) else { return false };
        // SAFETY: the loader's function takes the id by value.
        #[allow(unsafe_code)]
        let ready = unsafe { ready(AmmlTextureId::from(texture)) };
        ready != 0
    }

    /// Frees the texture at the next UI draw. A texture of an older generation, or one freed before, gives `Stale`.
    pub fn unload(&self, texture: TextureId) -> Result<(), ApiError> {
        let unload = self.table.and_then(|table| table.texture_unload).ok_or(ApiError::NotAvailable)?;
        // SAFETY: the module id lives for the call.
        #[allow(unsafe_code)]
        let status: AmmlStatus = unsafe { unload(AmmlStr::from_str(self.module), AmmlTextureId::from(texture)) };
        ApiError::check(status)
    }
}
