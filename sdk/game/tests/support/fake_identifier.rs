use amml_game::NetworkIdentifierLayout;

/// Builders for `NetworkIdentifier` values of each type.
pub struct FakeIdentifier;

impl FakeIdentifier {
    pub const RAKNET: i32 = 0;
    pub const ADDRESS: i32 = 1;
    pub const ADDRESS6: i32 = 2;
    pub const NETHER_NET: i32 = 3;
    pub const INVALID: i32 = 4;

    /// An all-zero identifier of the given type.
    pub fn blank(kind: i32) -> NetworkIdentifierLayout {
        // SAFETY: the mirror is plain integers and byte arrays; all zeroes is a valid value.
        let mut id: NetworkIdentifierLayout = unsafe { std::mem::zeroed() };
        id.kind = kind;
        id
    }

    pub fn raknet(guid: u64) -> NetworkIdentifierLayout {
        let mut id = Self::blank(Self::RAKNET);
        id.guid = guid;
        id
    }

    /// An IPv4 identifier: port at `sock` +2, address at +4.
    pub fn address(port: u16, address: [u8; 4]) -> NetworkIdentifierLayout {
        let mut id = Self::blank(Self::ADDRESS);
        id.sock[2..4].copy_from_slice(&port.to_le_bytes());
        id.sock[4..8].copy_from_slice(&address);
        id
    }

    /// An IPv6 identifier: port at `sock` +2, address at +8.
    pub fn address6(port: u16, address: [u8; 16]) -> NetworkIdentifierLayout {
        let mut id = Self::blank(Self::ADDRESS6);
        id.sock[2..4].copy_from_slice(&port.to_le_bytes());
        id.sock[8..24].copy_from_slice(&address);
        id
    }

    /// A NetherNet identifier with variant `index` and the 16 payload bytes.
    pub fn nether_net(index: u8, payload: [u8; 16]) -> NetworkIdentifierLayout {
        let mut id = Self::blank(Self::NETHER_NET);
        id.nether_net_id[..16].copy_from_slice(&payload);
        id.nether_net_id[0x10] = index;
        id
    }
}
