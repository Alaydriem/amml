use std::mem::offset_of;

use crate::game::{ItemInstanceLayout, ItemStackNetIdVariantLayout};

/// Layout mirror of the game's `ItemStack` (152 bytes): the `ItemStackBase` members, then the net id variant.
#[repr(C)]
pub struct ItemStackLayout {
    pub base: ItemInstanceLayout,                    // 0x00 ItemStackBase
    pub net_id_variant: ItemStackNetIdVariantLayout, // 0x80 mNetIdVariant
}

const _: () = assert!(offset_of!(ItemStackLayout, net_id_variant) == 0x80);
const _: () = assert!(size_of::<ItemStackLayout>() == 0x98);
