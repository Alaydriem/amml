use amml_game::ItemStackLayout;

use crate::support::GameMemory;

#[test]
fn reads_the_base_members_and_the_net_id_variant_at_the_item_stack_offsets() {
    let mut memory = GameMemory::new(0x98);
    // ItemStackBase::mAuxValue 0x20, mCount 0x22; ItemStack::mNetIdVariant 0x80 (its index byte at 0x90)
    memory.put::<i16>(0x20, 7);
    memory.put::<u8>(0x22, 12);
    memory.put::<i32>(0x80, 991);
    memory.put::<u8>(0x90, 0);

    // SAFETY: the buffer is 16-byte aligned, zeroed and at least as large as the mirror.
    let stack = unsafe { &*memory.ptr().cast::<ItemStackLayout>() };

    assert_eq!(stack.base.aux_value, 7);
    assert_eq!(stack.base.count, 12);
    assert_eq!(stack.net_id_variant.net_id(), Some(991));
}

#[test]
fn an_element_of_a_vector_of_item_stacks_starts_152_bytes_after_the_one_before() {
    let mut memory = GameMemory::new(0x98 * 2);
    memory.put::<u8>(0x98 + 0x22, 33);

    // SAFETY: the buffer is 16-byte aligned, zeroed and holds two mirrors.
    let stacks = unsafe { std::slice::from_raw_parts(memory.ptr().cast::<ItemStackLayout>(), 2) };

    assert_eq!(stacks[1].base.count, 33);
}
