use crate::ObjectRead;

/// The members of the game's `CreativeItemEntry` the creative pick reads, as offsets from the entry: the creative net
/// id (`mCreativeNetId`) and the `ItemInstance` (`mItemInstance`). The entries are 176 bytes apart in
/// `CreativeItemRegistry::mCreativeItems`. Read from the registry walk of the game on both gate builds: the stride
/// 0B0h and the item instance at +28h (26.63 `lea rcx,[rbx+28h]` ... `add rbx,0B0h`; 70.26 `lea rcx,[r14+28h]` ...
/// `add r14,0B0h`; survey section 7, row I1). The creative net id at +24h is proven by the exe on both builds (row I7).
pub struct CreativeItemEntryLayout;

impl CreativeItemEntryLayout {
    pub const SIZE: usize = 0xB0;
    pub const NET_ID_AT: usize = 0x24;
    pub const ITEM_INSTANCE_AT: usize = 0x28;

    /// The creative net id of the entry at `entry`.
    ///
    /// # Safety
    /// `entry` is a live entry of the creative registry.
    pub(crate) unsafe fn net_id(entry: usize) -> Option<u32> {
        // SAFETY: the caller guarantees a live entry; the net id lies inside its 0xB0 bytes.
        unsafe { ObjectRead::value::<u32>(entry, Self::NET_ID_AT) }
    }

    /// The address of the `ItemInstance` of the entry at `entry`.
    pub(crate) fn item_instance(entry: usize) -> usize {
        entry + Self::ITEM_INSTANCE_AT
    }
}
