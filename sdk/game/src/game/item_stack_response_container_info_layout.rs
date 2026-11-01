use std::mem::offset_of;

use amml_msvc::{MsvcVec, Optional};

use crate::game::ItemStackResponseSlotInfoLayout;

/// Layout mirror of the game's `ItemStackResponseContainerInfo` (40 bytes), one container of a response.
#[repr(C)]
pub struct ItemStackResponseContainerInfoLayout {
    pub container_name: u8,     // 0x00 mFullContainerName.mName (ContainerEnumName)
    _dynamic_id: Optional<u32>, // 0x04 mFullContainerName.mDynamicId
    pub slots: MsvcVec<ItemStackResponseSlotInfoLayout>, // 0x10 mSlots
}

const _: () = assert!(offset_of!(ItemStackResponseContainerInfoLayout, _dynamic_id) == 0x04);
const _: () = assert!(offset_of!(ItemStackResponseContainerInfoLayout, slots) == 0x10);
const _: () = assert!(size_of::<ItemStackResponseContainerInfoLayout>() == 0x28);
