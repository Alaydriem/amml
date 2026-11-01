use std::ffi::c_void;
use std::mem::offset_of;

/// Layout mirror of the game's `Packet` (48 bytes). A `ll::PayloadPacket<T>` is this, then `T` at 0x30, then the
/// packet's own `mSerializationMode` after `T`.
#[repr(C)]
pub struct PacketLayout {
    pub vtable: *const c_void, // 0x00
    _priority: i32,            // 0x08 mPriority
    _reliability: i32,         // 0x0C mReliability
    _sender_sub_id: u8,        // 0x10 mSenderSubId
    pub is_handled: bool,      // 0x11 mIsHandled
    _receive_timepoint: i64,   // 0x18 mReceiveTimepoint
    _handler: *const c_void,   // 0x20 mHandler
    _compressible: i32,        // 0x28 mCompressible
}

const _: () = assert!(offset_of!(PacketLayout, is_handled) == 0x11);
const _: () = assert!(offset_of!(PacketLayout, _compressible) == 0x28);
const _: () = assert!(size_of::<PacketLayout>() == 0x30);
