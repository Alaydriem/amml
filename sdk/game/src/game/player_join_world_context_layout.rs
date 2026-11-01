use std::mem::offset_of;

/// Layout mirror of the game's `PlayerJoinWorldContext` (176 bytes). The default value is what the game's default
/// constructor makes: an empty party id, three false flags, `ReconnectionType::None` and an empty telemetry optional.
/// The game destroys the context it is given, so a caller must not destroy it.
#[repr(C)]
pub struct PlayerJoinWorldContextLayout {
    _party_id: [u8; 0x20],          // 0x00 partyId (std::string)
    pub is_party_leader: bool,      // 0x20
    pub is_party_destination: bool, // 0x21
    pub is_server_transfer: bool,   // 0x22
    pub reconnection_type: u8,      // 0x23 (ReconnectionType: None 0)
    _pad: u32,                      // 0x24
    _telemetry: [u8; 0x80],         // 0x28 serverTelemetryData payload
    _telemetry_engaged: bool,       // 0xA8
    _tail: [u8; 7],                 // 0xA9
}

impl PlayerJoinWorldContextLayout {
    /// The default context: zeroes, with 0x0F (the small string capacity) at +0x18.
    pub fn default_value() -> Self {
        let mut party_id = [0u8; 0x20];
        party_id[0x18] = 0x0F;
        Self {
            _party_id: party_id,
            is_party_leader: false,
            is_party_destination: false,
            is_server_transfer: false,
            reconnection_type: 0,
            _pad: 0,
            _telemetry: [0; 0x80],
            _telemetry_engaged: false,
            _tail: [0; 7],
        }
    }
}

const _: () = assert!(offset_of!(PlayerJoinWorldContextLayout, is_party_leader) == 0x20);
const _: () = assert!(offset_of!(PlayerJoinWorldContextLayout, is_server_transfer) == 0x22);
const _: () = assert!(offset_of!(PlayerJoinWorldContextLayout, reconnection_type) == 0x23);
const _: () = assert!(offset_of!(PlayerJoinWorldContextLayout, _telemetry) == 0x28);
const _: () = assert!(offset_of!(PlayerJoinWorldContextLayout, _telemetry_engaged) == 0xA8);
const _: () = assert!(size_of::<PlayerJoinWorldContextLayout>() == 0xB0);
