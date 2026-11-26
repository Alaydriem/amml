use std::mem::offset_of;

/// Layout mirror of the MSVC `std::string_view`: the data pointer, then the length.
#[repr(C)]
pub struct StringViewLayout {
    data: *const u8, // 0x00
    len: usize,      // 0x08
}

impl StringViewLayout {
    /// The bytes of the view; empty for a null pointer.
    ///
    /// # Safety
    /// A non-null `data` points to `len` readable bytes that stay live while the slice is used.
    pub unsafe fn bytes(&self) -> &[u8] {
        if self.data.is_null() {
            return &[];
        }
        // SAFETY: the caller guarantees `len` readable bytes at `data`.
        unsafe { std::slice::from_raw_parts(self.data, self.len) }
    }
}

const _: () = assert!(offset_of!(StringViewLayout, len) == 0x08);
const _: () = assert!(size_of::<StringViewLayout>() == 0x10);
