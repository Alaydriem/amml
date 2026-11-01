use amml_game::ItemStackResponseSlotInfoLayout;

use crate::support::GameMemory;

#[test]
fn reads_the_slot_and_the_net_id_at_the_slot_info_offsets() {
    let mut memory = GameMemory::new(0x58 * 2);
    // ItemStackResponseSlotInfo (88 bytes): mSlot 0x01, mItemStackNetId 0x04
    memory.put::<u8>(0x58 + 0x01, 8);
    memory.put::<i32>(0x58 + 0x04, 1234);

    // SAFETY: the buffer is 16-byte aligned, zeroed and holds two mirrors.
    let slots = unsafe { std::slice::from_raw_parts(memory.ptr().cast::<ItemStackResponseSlotInfoLayout>(), 2) };

    assert_eq!(slots[1].slot, 8);
    assert_eq!(slots[1].item_stack_net_id, 1234);
}
