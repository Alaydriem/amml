use std::ffi::c_void;
use std::mem::offset_of;

use crate::game::ItemInstanceLayout;

/// Layout mirror of the game's `CreativeItemEntry` (176 bytes, the stride of `CreativeItemRegistry::mCreativeItems`).
/// The `Bedrock::EnableNonOwnerReferences` base is a vptr and a `std::shared_ptr`, 24 bytes.
#[repr(C)]
pub struct CreativeItemEntryLayout {
    _vtable: *const c_void,                // 0x00
    _control_block: [*const c_void; 2],    // 0x08 EnableNonOwnerReferences::mControlBlock
    _registry: *const c_void,              // 0x18 mRegistry
    _group_index: u32,                     // 0x20 mGroupIndex
    pub creative_net_id: u32,              // 0x24 mCreativeNetId (CreativeItemNetId: mRawId)
    pub item_instance: ItemInstanceLayout, // 0x28 mItemInstance
    _index: u32,                           // 0xA8 mIndex
}

const _: () = assert!(offset_of!(CreativeItemEntryLayout, creative_net_id) == 0x24);
const _: () = assert!(offset_of!(CreativeItemEntryLayout, item_instance) == 0x28);
const _: () = assert!(offset_of!(CreativeItemEntryLayout, _index) == 0xA8);
const _: () = assert!(size_of::<CreativeItemEntryLayout>() == 0xB0);
