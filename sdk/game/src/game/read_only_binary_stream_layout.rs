use std::ffi::c_void;
use std::mem::offset_of;

use amml_msvc::MsvcString;

/// Layout mirror of the game's `ReadOnlyBinaryStream` (72 bytes). `mView` (a `std::string_view`) is split into its
/// data pointer and length. With `copyBuffer`, the view points into `mOwnedBuffer`.
#[repr(C)]
pub struct ReadOnlyBinaryStreamLayout {
    pub vtable: *const c_void,    // 0x00
    pub owned_buffer: MsvcString, // 0x08 mOwnedBuffer
    pub view: *const u8,          // 0x28 mView data
    pub view_len: usize,          // 0x30 mView size
    pub read_pointer: u64,        // 0x38 mReadPointer
    pub has_overflowed: bool,     // 0x40 mHasOverflowed
}

const _: () = assert!(offset_of!(ReadOnlyBinaryStreamLayout, owned_buffer) == 0x08);
const _: () = assert!(offset_of!(ReadOnlyBinaryStreamLayout, view) == 0x28);
const _: () = assert!(offset_of!(ReadOnlyBinaryStreamLayout, view_len) == 0x30);
const _: () = assert!(offset_of!(ReadOnlyBinaryStreamLayout, read_pointer) == 0x38);
const _: () = assert!(offset_of!(ReadOnlyBinaryStreamLayout, has_overflowed) == 0x40);
const _: () = assert!(size_of::<ReadOnlyBinaryStreamLayout>() == 0x48);
