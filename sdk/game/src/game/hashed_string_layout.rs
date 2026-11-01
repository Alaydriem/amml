use std::ffi::c_void;
use std::mem::offset_of;

use amml_msvc::MsvcString;

/// Layout mirror of the game's `HashedString` (48 bytes): the hash, the text, then the last-match pointer.
#[repr(C)]
pub struct HashedStringLayout {
    pub hash: u64,              // 0x00 mStrHash
    pub text: MsvcString,       // 0x08 mStr
    _last_match: *const c_void, // 0x28 mLastMatch
}

impl HashedStringLayout {
    const OFFSET_BASIS: u64 = 0xCBF2_9CE4_8422_2325;
    const PRIME: u64 = 0x0100_0000_01B3;

    /// A `HashedString` that holds `text`, with the game's hash of it and no last match, as the game's
    /// `HashedString(std::string&&)` makes it. The caller keeps ownership of the text's heap block.
    pub fn new(text: MsvcString) -> Self {
        let hash = Self::hash_of(text.as_bytes());
        Self { hash, text, _last_match: std::ptr::null() }
    }

    /// The game's `HashedString::computeHash`: 0 for an empty text, else each byte, read as a signed `char`, is
    /// XORed into the hash after the multiply (FNV-1, not FNV-1a).
    pub fn hash_of(text: &[u8]) -> u64 {
        if text.is_empty() {
            return 0;
        }
        text.iter().fold(Self::OFFSET_BASIS, |hash, byte| (*byte as i8 as i64 as u64) ^ Self::PRIME.wrapping_mul(hash))
    }
}

const _: () = assert!(offset_of!(HashedStringLayout, text) == 0x08);
const _: () = assert!(size_of::<HashedStringLayout>() == 0x30);
