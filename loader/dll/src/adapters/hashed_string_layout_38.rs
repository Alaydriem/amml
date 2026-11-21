use std::ffi::c_void;
use std::mem::offset_of;

use amml_msvc::MsvcString;

use crate::StringViewLayout;

/// Layout mirror of the game's `HashedString` while `HashedString.layout` is 1 (70.x, 38h bytes): the hash, the
/// text as a `std::variant<std::string, std::string_view>` (the storage, then the index byte), then the last-match
/// pointer. The game's `operator==` reads the index at +28h and the last match at +30h.
#[repr(C)]
pub struct HashedStringLayout38 {
    pub hash: u64,              // 0x00 mStrHash
    storage: [u64; 4],          // 0x08 the variant storage: a std::string, or a std::string_view at its start
    pub index: u8,              // 0x28 the variant index: 0 the std::string, 1 the std::string_view
    _pad: [u8; 7],              // 0x29
    _last_match: *const c_void, // 0x30 mLastMatch
}

impl HashedStringLayout38 {
    pub const STRING: u8 = 0;
    pub const VIEW: u8 = 1;

    /// The text: the `std::string` or the `std::string_view` the index names; `None` for another index.
    ///
    /// # Safety
    /// The mirror is a live game `HashedString` of this layout, and a view points to `len` readable bytes.
    pub unsafe fn text(&self) -> Option<&[u8]> {
        match self.index {
            Self::STRING => {
                // SAFETY: index 0 holds a std::string at the start of the storage, which has its size and alignment.
                let text = unsafe { &*self.storage.as_ptr().cast::<MsvcString>() };
                Some(text.as_bytes())
            }
            Self::VIEW => {
                // SAFETY: index 1 holds a std::string_view at the start of the storage.
                let view = unsafe { &*self.storage.as_ptr().cast::<StringViewLayout>() };
                // SAFETY: the caller guarantees the view's bytes are readable.
                Some(unsafe { view.bytes() })
            }
            _ => None,
        }
    }
}

const _: () = assert!(offset_of!(HashedStringLayout38, storage) == 0x08);
const _: () = assert!(offset_of!(HashedStringLayout38, index) == 0x28);
const _: () = assert!(size_of::<HashedStringLayout38>() == 0x38);
const _: () = assert!(size_of::<MsvcString>() == 0x20);
