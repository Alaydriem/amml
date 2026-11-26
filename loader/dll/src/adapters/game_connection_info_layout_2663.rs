use std::mem::offset_of;

use amml_msvc::MsvcString;

/// Layout mirror of the head of the game's `Social::GameConnectionInfo` on 26.63 (the value is 2F0h bytes there), up
/// to and with `mPort`.
#[repr(C)]
pub struct GameConnectionInfoLayout2663 {
    pub connection_type: i16, // 0x00 mType (Social::ConnectionType)
    _transport: i32,          // 0x04 mTransportType
    pub host: MsvcString,     // 0x08 mHostIpAddress
    _names: [u8; 0x64],       // 0x28 mUnresolvedUrl, mServerRegion, mExternalServerName, mServiceQuality
    pub port: i32,            // 0x8C mPort
}

const _: () = assert!(offset_of!(GameConnectionInfoLayout2663, host) == 0x08);
const _: () = assert!(offset_of!(GameConnectionInfoLayout2663, port) == 0x8C);
const _: () = assert!(size_of::<GameConnectionInfoLayout2663>() == 0x90);
