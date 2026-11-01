use std::ffi::c_void;
use std::mem::offset_of;

/// Layout mirror of the game's `Dimension`, up to and with `mSkyDarken`. The same on 26.51 and 26.60.
#[repr(C)]
pub struct DimensionLayout {
    _vtable: *const c_void, // 0x000
    _members: [u8; 0x1A1],  // 0x008 IDimension and the members up to mHasSkylight (0x1A8)
    pub sky_darken: u8,     // 0x1A9 mSkyDarken
}

const _: () = assert!(offset_of!(DimensionLayout, sky_darken) == 0x1A9);
