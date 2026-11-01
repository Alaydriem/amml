use amml_msvc::MsvcVariant;

/// Layout mirror of the game's `ItemStackNetIdVariant` (24 bytes): a
/// `std::variant<ItemStackNetId, ItemStackRequestId, ItemStackLegacyRequestId>`. The two request ids have a vptr, so
/// the storage is 16 bytes. The net id (index 0) is a plain `int` at offset 0.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct ItemStackNetIdVariantLayout(MsvcVariant<[u64; 2]>);

impl ItemStackNetIdVariantLayout {
    const NET_ID_INDEX: u8 = 0;

    /// A variant that holds the net id `net_id`, as the game's assignment of an `ItemStackNetId` makes it.
    pub fn from_net_id(net_id: i32) -> Self {
        Self(MsvcVariant::new(Self::NET_ID_INDEX, [u64::from(net_id as u32), 0]))
    }

    /// The net id, or `None` when the variant holds a request id.
    pub fn net_id(&self) -> Option<i32> {
        if self.0.index() != Self::NET_ID_INDEX {
            return None;
        }
        Some(self.0.storage()[0] as u32 as i32)
    }
}

const _: () = assert!(size_of::<ItemStackNetIdVariantLayout>() == 0x18);
