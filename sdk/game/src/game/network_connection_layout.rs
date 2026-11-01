use std::ffi::c_void;
use std::mem::offset_of;

use crate::game::NetworkIdentifierLayout;

/// Layout mirror of the head of the game's `NetworkConnection`, up to and with `mPeer`. The whole object is 0x1B0 bytes
/// on 26.51 and 0x1E0 on 26.60, so it is a prefix: use it only through a pointer to a game connection, never by value.
#[repr(C)]
pub struct NetworkConnectionLayout {
    pub id: NetworkIdentifierLayout, // 0x00 mId
    pub kind: i32,                   // 0xB0 mType
    _between: [u8; 0x64],            // 0xB4
    pub peer: *mut c_void,           // 0x118 mPeer
}

const _: () = assert!(offset_of!(NetworkConnectionLayout, id) == 0x00);
const _: () = assert!(offset_of!(NetworkConnectionLayout, kind) == 0xB0);
const _: () = assert!(offset_of!(NetworkConnectionLayout, peer) == 0x118);
