use crate::AmmlStatus;

/// A borrowed UTF-8 string view that crosses the C boundary.
///
/// The bytes are not NUL-terminated. They belong to the sender and stay valid for the duration of the call that carries the view.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AmmlStr {
    pub ptr: *const u8,
    pub len: usize,
}

impl AmmlStr {
    /// A view of no bytes.
    pub const EMPTY: Self = Self { ptr: core::ptr::null(), len: 0 };

    /// Borrows a Rust string. The view is valid while `text` is.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(text: &str) -> Self {
        Self { ptr: text.as_ptr(), len: text.len() }
    }

    /// Borrows bytes after checking they are UTF-8; invalid UTF-8 gives `InvalidArgument`.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, AmmlStatus> {
        match core::str::from_utf8(bytes) {
            Ok(text) => Ok(Self::from_str(text)),
            Err(_) => Err(AmmlStatus::INVALID_ARGUMENT),
        }
    }

    /// Reads the view as text. A null pointer with a non-zero length, or bytes that are not UTF-8,
    /// give `InvalidArgument`.
    ///
    /// # Safety
    /// `ptr` must be valid for reads of `len` bytes for the lifetime chosen by the caller.
    #[allow(unsafe_code)]
    pub unsafe fn as_str<'a>(&self) -> Result<&'a str, AmmlStatus> {
        if self.len == 0 {
            return Ok("");
        }
        if self.ptr.is_null() {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        // SAFETY: the caller guarantees `ptr` is valid for `len` bytes; `ptr` is non-null and `len` non-zero here.
        let bytes = unsafe { core::slice::from_raw_parts(self.ptr, self.len) };
        core::str::from_utf8(bytes).map_err(|_| AmmlStatus::INVALID_ARGUMENT)
    }
}
