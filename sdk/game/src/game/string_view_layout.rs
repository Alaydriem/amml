use std::mem::offset_of;

/// Layout mirror of the MSVC `std::string_view` (16 bytes): the data pointer, then the length.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct StringViewLayout {
    data: *const u8, // 0x00
    len: usize,      // 0x08
}

impl StringViewLayout {
    /// A view of `text`. The view does not own the bytes: they must outlive every use of it.
    pub fn of(text: &str) -> Self {
        Self { data: text.as_ptr(), len: text.len() }
    }
}

const _: () = assert!(offset_of!(StringViewLayout, len) == 0x08);
const _: () = assert!(size_of::<StringViewLayout>() == 16);
