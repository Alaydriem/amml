/// The game's per-type `NetworkIdentifier` compare (`equalsTypeData`, called by `NetworkSystem::_sendInternal` for each
/// connection after it compared the two types: 26.63 0x2375400, 70.26 0x301fa00), on the B0h bytes of two
/// identifiers. Ids of different types (`mType` at A8h) are never equal. Within a type only the fields the game compares
/// count: RakNet (0) the guid at 18h; Address (1) the port and the 4 address bytes of the socket address at 28h (+2,
/// +4); Address6 (2) the port and the 16 address bytes (+2, +8); NetherNet (3) the variant index at 10h, then 8 bytes
/// (index 1) or 16 bytes (index 2) at 0, any other index equal; Invalid (4) always equal; any other type never equal.
/// The same rule as `amml-game`'s `NetworkIdentifierEquality`; the loader does not depend on `amml-game`, so a change
/// there must be made here too.
pub struct NetworkIdEquality;

impl NetworkIdEquality {
    /// The size of a `NetworkIdentifier`.
    pub const SIZE: usize = 0xB0;
    const KIND_AT: usize = 0xA8;
    const GUID_AT: usize = 0x18;
    const SOCK_AT: usize = 0x28;
    const NETHER_INDEX_AT: usize = 0x10;
    const RAKNET: i32 = 0;
    const ADDRESS: i32 = 1;
    const ADDRESS6: i32 = 2;
    const NETHER_NET: i32 = 3;
    const INVALID: i32 = 4;

    /// Whether the game treats the identifiers `a` and `b` as the same peer.
    pub fn equals(a: &[u8; Self::SIZE], b: &[u8; Self::SIZE]) -> bool {
        let kind = Self::kind(a);
        if kind != Self::kind(b) {
            return false;
        }
        let same = |at: usize, len: usize| a[at..at + len] == b[at..at + len];
        match kind {
            Self::RAKNET => same(Self::GUID_AT, 8),
            Self::ADDRESS => same(Self::SOCK_AT + 2, 2) && same(Self::SOCK_AT + 4, 4),
            Self::ADDRESS6 => same(Self::SOCK_AT + 2, 2) && same(Self::SOCK_AT + 8, 16),
            Self::NETHER_NET => {
                let index = a[Self::NETHER_INDEX_AT];
                index == b[Self::NETHER_INDEX_AT]
                    && match index {
                        1 => same(0, 8),
                        2 => same(0, 16),
                        _ => true,
                    }
            }
            Self::INVALID => true,
            _ => false,
        }
    }

    fn kind(id: &[u8; Self::SIZE]) -> i32 {
        i32::from_le_bytes([id[Self::KIND_AT], id[Self::KIND_AT + 1], id[Self::KIND_AT + 2], id[Self::KIND_AT + 3]])
    }
}
