use std::ffi::c_void;
use std::mem::offset_of;

/// Prefix mirror of the game's `BaseActorRenderContext` (26.60): the members before and including the item
/// renderer. The size of the whole object is the `BaseActorRenderContext.size` entry.
#[repr(C)]
pub struct BaseActorRenderContextLayout {
    _head: [u8; 0x58],              // 0x00 the vtable and the members before mItemRenderer
    pub item_renderer: *mut c_void, // 0x58 mItemRenderer (ItemRenderer&)
}

impl BaseActorRenderContextLayout {
    /// The smallest object that holds `item_renderer`.
    pub const MIN_SIZE: usize = size_of::<Self>();
}

const _: () = assert!(offset_of!(BaseActorRenderContextLayout, item_renderer) == 0x58);
