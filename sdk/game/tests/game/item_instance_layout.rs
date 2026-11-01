use std::ffi::c_void;

use amml_game::ItemInstanceLayout;

use crate::support::GameMemory;

fn view(memory: &mut GameMemory) -> &ItemInstanceLayout {
    // SAFETY: the buffer is 16-byte aligned, zeroed and at least as large as the mirror.
    unsafe { &*memory.ptr().cast::<ItemInstanceLayout>() }
}

#[test]
fn reads_the_aux_value_and_the_count_at_the_item_stack_base_offsets() {
    let mut memory = GameMemory::new(0x80);
    // ItemStackBase::mAuxValue (short) at 0x20, mCount (uchar) at 0x22
    memory.put::<i16>(0x20, -3);
    memory.put::<u8>(0x22, 64);

    let item = view(&mut memory);

    assert_eq!(item.aux_value, -3);
    assert_eq!(item.count, 64);
}

#[test]
#[allow(unsafe_code)]
fn item_follows_the_weak_ptr_counter_to_the_item() {
    // SharedCounter<Item>: the Item pointer is its first member
    let mut counter = GameMemory::new(0x10);
    counter.put::<usize>(0, 0x1234_5678);
    let mut memory = GameMemory::new(0x80);
    // ItemStackBase::mItem (WeakPtr<Item>, one SharedCounter pointer) at 0x08
    memory.put::<usize>(0x08, counter.ptr() as usize);

    let item = view(&mut memory);

    // SAFETY: the counter pointer points to a live fake counter.
    assert_eq!(unsafe { item.item() }, 0x1234_5678 as *const c_void);
}

#[test]
#[allow(unsafe_code)]
fn item_is_null_when_the_weak_ptr_has_no_counter() {
    let mut memory = GameMemory::new(0x80);

    let item = view(&mut memory);

    // SAFETY: a null counter is not read.
    assert!(unsafe { item.item() }.is_null());
}
