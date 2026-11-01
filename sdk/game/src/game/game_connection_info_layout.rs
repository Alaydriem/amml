use std::mem::offset_of;

use amml_msvc::MsvcString;

/// Layout mirror of the head of the game's `Social::GameConnectionInfo`, up to and with `mPort`. The head is the same
/// on 26.51 and 26.60; the tail is not, so the size of the whole value comes from the `GameConnectionInfo.size`
/// entry.
#[repr(C)]
pub struct GameConnectionInfoLayout {
    pub connection_type: i16, // 0x00 mType (Social::ConnectionType)
    _transport: i32,          // 0x04 mTransportType
    pub host: MsvcString,     // 0x08 mHostIpAddress
    pub url: MsvcString,      // 0x28 mUnresolvedUrl
    _names: [u8; 0x40],       // 0x48 mServerRegion, mExternalServerName
    _service_quality: i32,    // 0x88 mServiceQuality
    pub port: i32,            // 0x8C mPort
}

const _: () = assert!(offset_of!(GameConnectionInfoLayout, host) == 0x08);
const _: () = assert!(offset_of!(GameConnectionInfoLayout, url) == 0x28);
const _: () = assert!(offset_of!(GameConnectionInfoLayout, port) == 0x8C);
const _: () = assert!(size_of::<GameConnectionInfoLayout>() == 0x90);
