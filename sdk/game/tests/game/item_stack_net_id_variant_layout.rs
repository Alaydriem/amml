use amml_game::ItemStackNetIdVariantLayout;

use crate::support::GameMemory;

fn view(memory: &mut GameMemory) -> &ItemStackNetIdVariantLayout {
    // SAFETY: the buffer is 16-byte aligned, zeroed and at least as large as the mirror.
    unsafe { &*memory.ptr().cast::<ItemStackNetIdVariantLayout>() }
}

#[test]
fn net_id_reads_the_int_at_offset_zero_when_the_index_is_zero() {
    let mut memory = GameMemory::new(0x18);
    // variant<ItemStackNetId, ItemStackRequestId, ItemStackLegacyRequestId>: storage 16 bytes, index byte at 0x10
    memory.put::<i32>(0x00, -77);
    memory.put::<u8>(0x10, 0);

    assert_eq!(view(&mut memory).net_id(), Some(-77));
}

#[test]
fn net_id_is_none_for_a_request_id() {
    let mut memory = GameMemory::new(0x18);
    // ItemStackRequestId has a vptr, so its mRawId is at 0x08
    memory.put::<usize>(0x00, 0xDEAD_0000);
    memory.put::<i32>(0x08, 5);
    memory.put::<u8>(0x10, 1);

    assert_eq!(view(&mut memory).net_id(), None);
}

#[test]
fn from_net_id_writes_index_zero_and_the_int_at_offset_zero() {
    let mut memory = GameMemory::new(0x18);
    memory.put::<u8>(0x10, 2);

    memory.put(0x00, ItemStackNetIdVariantLayout::from_net_id(42));

    assert_eq!(memory.get::<i32>(0x00), 42);
    assert_eq!(memory.get::<u8>(0x10), 0);
}
