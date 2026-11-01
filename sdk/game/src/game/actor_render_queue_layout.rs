use std::ffi::c_void;
use std::mem::offset_of;

/// Layout mirror of the game's `brstd::flat_map<not_null<Actor*>, ShadowContext>` (48 bytes): two MSVC vectors side by
/// side. The keys are 8 bytes each, the values 1 byte each. The pointers belong to the game allocator: never use them
/// as a `Vec`, never write a `cap` and never reallocate.
#[repr(C)]
pub struct ActorRenderQueueLayout {
    pub keys_begin: *mut *mut c_void, // 0x00
    pub keys_end: *mut *mut c_void,   // 0x08
    pub keys_cap: *mut *mut c_void,   // 0x10
    pub values_begin: *mut u8,        // 0x18
    pub values_end: *mut u8,          // 0x20
    pub values_cap: *mut u8,          // 0x28
}

const _: () = assert!(offset_of!(ActorRenderQueueLayout, keys_end) == 0x08);
const _: () = assert!(offset_of!(ActorRenderQueueLayout, keys_cap) == 0x10);
const _: () = assert!(offset_of!(ActorRenderQueueLayout, values_begin) == 0x18);
const _: () = assert!(offset_of!(ActorRenderQueueLayout, values_end) == 0x20);
const _: () = assert!(offset_of!(ActorRenderQueueLayout, values_cap) == 0x28);
const _: () = assert!(size_of::<ActorRenderQueueLayout>() == 0x30);
