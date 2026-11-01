use std::ffi::c_void;
use std::mem::offset_of;

/// Prefix mirror of the game's `ScreenView` (26.60): the members the peek reads. `mSize` at 0x18 is a header value
/// that was not isolated in the executable; it is checked in the running game.
#[repr(C)]
pub struct ScreenViewLayout {
    _head: [u8; 0x18],                // 0x00 mLastTime, mTickTime and the members before mSize
    pub size: [f32; 2],               // 0x18 mSize (glm::vec2)
    _members: [u8; 0x20],             // 0x20 the members between mSize and mController
    pub controller: *mut c_void,      // 0x40 mController: the raw pointer of the shared_ptr<ScreenController>
    _controller_control: *mut c_void, // 0x48 mController: the control block
}

const _: () = assert!(offset_of!(ScreenViewLayout, size) == 0x18);
const _: () = assert!(offset_of!(ScreenViewLayout, controller) == 0x40);
const _: () = assert!(size_of::<ScreenViewLayout>() == 0x50);
