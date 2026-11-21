use std::ffi::c_void;
use std::mem::offset_of;

use amml_msvc::MsvcString;

/// Layout mirror of the game's `HashedString` while `HashedString.layout` is 0 (26.63, 30h bytes): the hash, the
/// text as a `std::string`, then the last-match pointer.
#[repr(C)]
pub struct HashedStringLayout30 {
    pub hash: u64,              // 0x00 mStrHash
    pub text: MsvcString,       // 0x08 mStr
    _last_match: *const c_void, // 0x28 mLastMatch
}

const _: () = assert!(offset_of!(HashedStringLayout30, text) == 0x08);
const _: () = assert!(size_of::<HashedStringLayout30>() == 0x30);
