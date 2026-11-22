use std::ffi::c_void;
use std::mem::offset_of;

/// Layout mirror of the game's `StringByteInput` (20h bytes on 26.63 and 70.26): an `IDataInput` over a
/// `std::string_view`, read from `index` on. The loader builds one in its own memory for `NbtIo::read`, with the
/// vtable of `StringByteInput.vtable`, as the game builds one inline: vtable, index 0, data, size.
#[repr(C)]
pub struct StringByteInputLayout {
    pub vtable: *const c_void, // 0x00
    pub index: usize,          // 0x08 mIdx
    pub data: *const u8,       // 0x10 the data of mBuffer
    pub len: usize,            // 0x18 the size of mBuffer
}

impl StringByteInputLayout {
    /// An input over `bytes` from the start. `bytes` must outlive every read through it.
    pub fn new(vtable: usize, bytes: &[u8]) -> Self {
        Self { vtable: std::ptr::with_exposed_provenance(vtable), index: 0, data: bytes.as_ptr(), len: bytes.len() }
    }
}

const _: () = assert!(offset_of!(StringByteInputLayout, index) == 0x08);
const _: () = assert!(offset_of!(StringByteInputLayout, data) == 0x10);
const _: () = assert!(size_of::<StringByteInputLayout>() == 0x20);
