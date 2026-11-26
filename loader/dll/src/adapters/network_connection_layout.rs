use std::ffi::c_void;
use std::mem::offset_of;

/// Layout mirror of the head of the game's `NetworkConnection`, up to and with `mPeer`: a prefix, used only through a
/// pointer to a game connection. Read from the exe on 26.63 and 70.26 (survey section 8, rows C2 and N7): `mId` (the
/// B0h-byte `NetworkIdentifier`) at 0, `mType` (a dword, 0 remote, 1 local) at B0h, `mPeer` (the `shared_ptr` pointer)
/// at 118h. The same mirror as `amml-game`'s `NetworkConnectionLayout` (`sdk/game/src/game/network_connection_layout.rs`);
/// the loader does not depend on `amml-game`, so a change there must be made here too. The wire adapter checks the
/// derived `NetworkConnection.mType` and `NetworkConnection.mPeer` against it before it offers its events.
#[repr(C)]
pub struct NetworkConnectionLayout {
    pub id: [u8; 0xB0],    // 0x00 mId
    pub kind: i32,         // 0xB0 mType
    _between: [u8; 0x64],  // 0xB4
    pub peer: *mut c_void, // 0x118 mPeer
}

impl NetworkConnectionLayout {
    /// The offset of `mType`.
    pub const TYPE_AT: usize = offset_of!(NetworkConnectionLayout, kind);
    /// The offset of `mPeer`.
    pub const PEER_AT: usize = offset_of!(NetworkConnectionLayout, peer);
}

const _: () = assert!(offset_of!(NetworkConnectionLayout, id) == 0x00);
const _: () = assert!(NetworkConnectionLayout::TYPE_AT == 0xB0);
const _: () = assert!(NetworkConnectionLayout::PEER_AT == 0x118);
