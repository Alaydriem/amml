use std::mem::offset_of;

/// Layout mirror of the game's `NetworkIdentifier` (176 bytes). The game compares only the fields of the active
/// `kind`: RakNet the `guid`; Address and Address6 the port (`sock` +2) and the address (`sock` +4, or +8 for 16
/// bytes); NetherNet the variant index at `nether_net_id` +0x10, then its 8 or 16 bytes at +0; Invalid nothing.
#[repr(C)]
pub struct NetworkIdentifierLayout {
    pub nether_net_id: [u8; 0x18], // 0x00 mNetherNetId (variant: payload +0, index +0x10)
    pub guid: u64,                 // 0x18 mGuid.g
    _guid_system_index: [u8; 8],   // 0x20 mGuid.systemIndex
    pub sock: [u8; 0x80],          // 0x28 mSock (sockaddr_in / sockaddr_in6)
    pub kind: i32,                 // 0xA8 mType (RakNet 0, Address 1, Address6 2, NetherNet 3, Invalid 4)
}

const _: () = assert!(offset_of!(NetworkIdentifierLayout, guid) == 0x18);
const _: () = assert!(offset_of!(NetworkIdentifierLayout, sock) == 0x28);
const _: () = assert!(offset_of!(NetworkIdentifierLayout, kind) == 0xA8);
const _: () = assert!(size_of::<NetworkIdentifierLayout>() == 0xB0);
