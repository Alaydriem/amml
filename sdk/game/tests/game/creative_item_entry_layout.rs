use amml_game::CreativeItemEntryLayout;

use crate::support::GameMemory;

#[test]
fn reads_the_net_id_and_the_item_instance_after_the_non_owner_base() {
    let mut memory = GameMemory::new(0xB0 * 2);
    // CreativeItemEntry (176 bytes): EnableNonOwnerReferences (vptr, shared_ptr) 0x00..0x18, mRegistry 0x18,
    // mGroupIndex 0x20, mCreativeNetId 0x24, mItemInstance 0x28 (its mAuxValue at +0x20, mCount at +0x22)
    let second = 0xB0;
    memory.put::<u32>(second + 0x24, 812);
    memory.put::<i16>(second + 0x28 + 0x20, 2);
    memory.put::<u8>(second + 0x28 + 0x22, 1);

    // SAFETY: the buffer is 16-byte aligned, zeroed and holds two mirrors.
    let entries = unsafe { std::slice::from_raw_parts(memory.ptr().cast::<CreativeItemEntryLayout>(), 2) };

    assert_eq!(entries[1].creative_net_id, 812);
    assert_eq!(entries[1].item_instance.aux_value, 2);
    assert_eq!(entries[1].item_instance.count, 1);
}
