use crate::{AmmlEvent, AmmlEventHeader, AmmlTextureReload};

/// The game dropped its textures and the loader handled the mod textures (`reason` says how): a mod that keeps pixels
/// of its own uploads them again here.
///
/// Raised on thread `GAME`, phase After, at most once per reason and frame, after the loader bound the textures again
/// (`RESOURCE_PACK`) or freed them (`DEVICE_RESET`).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct TexturesReloaded {
    pub header: AmmlEventHeader,
    pub reason: AmmlTextureReload,
    pub _reserved: [u8; 7],
}

impl AmmlEvent for TexturesReloaded {
    const ID: &'static str = "ui.textures_reloaded";
}

const _: () = assert!(core::mem::size_of::<TexturesReloaded>() == 24);
