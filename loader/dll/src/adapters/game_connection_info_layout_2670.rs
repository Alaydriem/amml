use std::mem::offset_of;

use amml_msvc::MsvcString;

/// Layout mirror of the head of the game's `Social::GameConnectionInfo` on 70.26 (the value is 308h bytes there), up
/// to and with `mPort`. Every member after the type sits 18h bytes later than on 26.63.
#[repr(C)]
pub struct GameConnectionInfoLayout2670 {
    pub connection_type: i16, // 0x00 mType (Social::ConnectionType)
    _before_host: [u8; 0x1E], // 0x02 mTransportType and the members the 70.26 build adds
    pub host: MsvcString,     // 0x20 mHostIpAddress
    _names: [u8; 0x64],       // 0x40 mUnresolvedUrl, mServerRegion, mExternalServerName, mServiceQuality
    pub port: i32,            // 0xA4 mPort
}

const _: () = assert!(offset_of!(GameConnectionInfoLayout2670, host) == 0x20);
const _: () = assert!(offset_of!(GameConnectionInfoLayout2670, port) == 0xA4);
const _: () = assert!(size_of::<GameConnectionInfoLayout2670>() == 0xA8);
