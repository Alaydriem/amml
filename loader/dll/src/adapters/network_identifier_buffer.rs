/// A `NetworkIdentifier` (B0h bytes) the loader owns, passed to a client network handler slot as the sender of a packet
/// the loader makes: every byte zero but the type `mType` at A8h, which is 4, the invalid type. Read from the exe on
/// 26.63 and 70.26: the game's per-type identifier compare (26.63 0x2375400, 70.26 0x301fa00, called by
/// `NetworkSystem::_sendInternal` after it compared the two types) switches on `[id+0A8h]` over 0..4 (`ja` to false for
/// more); case 0 compares the RakNet guid at +18h, and case 4 compares nothing but the other type against 4. So type 4
/// carries no peer data and an identifier of type 4 never names a real peer (an all-zero one would be a RakNet peer with
/// guid 0). No game code reads it: the `CameraInstructionPacket` and `PrimitiveShapesPacket` handlers never read their
/// identifier on 26.63 and 70.26 (survey sections 8.11 and 8.12). A hook after the loader's on such a slot reads
/// readable bytes of the right size and sees an invalid sender.
///
/// The size and the place of `mType` are those of `amml-game`'s `NetworkIdentifierLayout`
/// (`sdk/game/src/game/network_identifier_layout.rs`: size B0h and `kind` at A8h asserted there; Invalid 4 in its doc
/// and in `NetworkIdentifierEquality`); the loader does not depend on `amml-game`, so a change there must be made here
/// too.
#[repr(C, align(8))]
pub struct NetworkIdentifierBuffer {
    _head: [u8; 0xA8], // 0x00 mNetherNetId, mGuid, mSock
    _kind: i32,        // 0xA8 mType
    _tail: [u8; 4],    // 0xAC padding
}

impl NetworkIdentifierBuffer {
    /// The size of the game's `NetworkIdentifier`.
    pub const SIZE: usize = 0xB0;
    /// `mType` of an invalid identifier.
    pub const INVALID: i32 = 4;

    pub fn new() -> Self {
        Self { _head: [0; 0xA8], _kind: Self::INVALID, _tail: [0; 4] }
    }

    /// The address to pass as the handler's identifier argument; valid while `self` lives.
    pub fn as_mut_ptr(&mut self) -> *mut std::ffi::c_void {
        std::ptr::from_mut(self).cast()
    }
}

impl Default for NetworkIdentifierBuffer {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = assert!(std::mem::offset_of!(NetworkIdentifierBuffer, _kind) == 0xA8);
const _: () = assert!(size_of::<NetworkIdentifierBuffer>() == NetworkIdentifierBuffer::SIZE);
