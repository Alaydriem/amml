use crate::game::NetworkIdentifierLayout;

/// The game's per-type `NetworkIdentifier` compare (`equalsTypeData`), which does not compare all 0xB0 bytes.
pub struct NetworkIdentifierEquality;

impl NetworkIdentifierEquality {
    // `mType` of a RakNet id.
    const RAKNET: i32 = 0;
    // `mType` of an IPv4 id.
    const ADDRESS: i32 = 1;
    // `mType` of an IPv6 id.
    const ADDRESS6: i32 = 2;
    // `mType` of a NetherNet id.
    const NETHER_NET: i32 = 3;
    // `mType` of an invalid id.
    const INVALID: i32 = 4;

    pub fn new() -> Self {
        Self
    }

    /// Whether the game treats `a` and `b` as the same peer. Ids of different types are never equal. Within a type
    /// only the fields the game compares count:
    /// - RakNet: the guid;
    /// - Address: the port (`sock` +2) and the 4 address bytes (`sock` +4);
    /// - Address6: the port and the 16 address bytes (`sock` +8);
    /// - NetherNet: the variant index at `nether_net_id` +0x10, then 8 bytes (index 1) or 16 bytes (index 2) at +0;
    ///   index 0 and any index above 2 are equal;
    /// - Invalid: always equal;
    /// - any other type: never equal.
    pub fn equals(&self, a: &NetworkIdentifierLayout, b: &NetworkIdentifierLayout) -> bool {
        if a.kind != b.kind {
            return false;
        }
        match a.kind {
            Self::RAKNET => a.guid == b.guid,
            Self::ADDRESS => a.sock[2..4] == b.sock[2..4] && a.sock[4..8] == b.sock[4..8],
            Self::ADDRESS6 => a.sock[2..4] == b.sock[2..4] && a.sock[8..24] == b.sock[8..24],
            Self::NETHER_NET => Self::nether_net_equals(a, b),
            Self::INVALID => true,
            _ => false,
        }
    }

    fn nether_net_equals(a: &NetworkIdentifierLayout, b: &NetworkIdentifierLayout) -> bool {
        let index = a.nether_net_id[0x10];
        if index != b.nether_net_id[0x10] {
            return false;
        }
        match index {
            1 => a.nether_net_id[..8] == b.nether_net_id[..8],
            2 => a.nether_net_id[..16] == b.nether_net_id[..16],
            _ => true,
        }
    }
}

impl Default for NetworkIdentifierEquality {
    fn default() -> Self {
        Self::new()
    }
}
