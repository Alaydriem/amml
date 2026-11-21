use amml_abi::TexturesReloaded;

use crate::TextureReload;

/// Typed reads of a `TexturesReloaded`.
pub trait TexturesReloadedExt {
    fn reason(&self) -> TextureReload;
}

impl TexturesReloadedExt for TexturesReloaded {
    fn reason(&self) -> TextureReload {
        TextureReload::from(self.reason)
    }
}
