use std::ffi::c_void;
use std::mem::offset_of;

/// Layout mirror of the game's `Block`, up to and with `mBlockType`. The same on 26.51 and 26.60.
#[repr(C)]
pub struct BlockLayout {
    _vtable: *const c_void,      // 0x00
    _components: [u64; 12],      // 0x08 mComponents
    pub block_type: *mut c_void, // 0x68 mBlockType
}

const _: () = assert!(offset_of!(BlockLayout, block_type) == 0x68);
