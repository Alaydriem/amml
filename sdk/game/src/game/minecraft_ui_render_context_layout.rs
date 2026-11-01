use std::ffi::c_void;
use std::mem::offset_of;

/// Prefix mirror of the game's `MinecraftUIRenderContext` (26.60): the vtable and the client. `mScreenContext` is the
/// `MinecraftUIRenderContext.mScreenContext` entry.
#[repr(C)]
pub struct MinecraftUiRenderContextLayout {
    _vtable: *const c_void,  // 0x00
    pub client: *mut c_void, // 0x08 mClient (IClientInstance*)
}

const _: () = assert!(offset_of!(MinecraftUiRenderContextLayout, client) == 0x08);
