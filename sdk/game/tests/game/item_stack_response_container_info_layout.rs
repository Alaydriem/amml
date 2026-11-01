use amml_game::ItemStackResponseContainerInfoLayout;

use crate::support::GameMemory;

#[test]
fn reads_the_container_name_and_the_slots_at_the_container_info_offsets() {
    let mut slots = GameMemory::new(0x58 * 2);
    slots.put::<u8>(0x01, 3);
    slots.put::<u8>(0x58 + 0x01, 4);
    let first = slots.ptr() as usize;
    let mut memory = GameMemory::new(0x28 * 2);
    // ItemStackResponseContainerInfo (40 bytes): mFullContainerName.mName 0x00, mSlots (vector) 0x10
    memory.put::<u8>(0x28, 29);
    memory.put::<[usize; 3]>(0x28 + 0x10, [first, first + 0x58 * 2, first + 0x58 * 2]);

    // SAFETY: the buffer is 16-byte aligned and holds two mirrors; the second one's vector points to two live slots.
    let containers =
        unsafe { std::slice::from_raw_parts(memory.ptr().cast::<ItemStackResponseContainerInfoLayout>(), 2) };

    assert_eq!(containers[1].container_name, 29);
    let read: Vec<u8> = containers[1].slots.as_slice().iter().map(|slot| slot.slot).collect();
    assert_eq!(read, [3, 4]);
}
