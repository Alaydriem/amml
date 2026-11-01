use std::mem::offset_of;

/// Layout mirror of the game's `ItemStackResponseSlotInfo` (88 bytes), one slot of a container in an
/// `ItemStackResponsePacket`.
#[repr(C)]
pub struct ItemStackResponseSlotInfoLayout {
    _requested_slot: u8,         // 0x00 mRequestedSlot
    pub slot: u8,                // 0x01 mSlot
    _amount: u8,                 // 0x02 mAmount
    pub item_stack_net_id: i32,  // 0x04 mItemStackNetId (ItemStackNetId: a plain int)
    _custom_name: [u64; 9],      // 0x08 mCustomName (Bedrock::Safety::RedactableString)
    _durability_correction: i16, // 0x50 mDurabilityCorrection
}

const _: () = assert!(offset_of!(ItemStackResponseSlotInfoLayout, slot) == 0x01);
const _: () = assert!(offset_of!(ItemStackResponseSlotInfoLayout, item_stack_net_id) == 0x04);
const _: () = assert!(offset_of!(ItemStackResponseSlotInfoLayout, _durability_correction) == 0x50);
const _: () = assert!(size_of::<ItemStackResponseSlotInfoLayout>() == 0x58);
