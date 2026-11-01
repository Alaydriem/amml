use std::ffi::c_void;

use amml_testmod::{LayoutFault, NameTagLayout};

fn layout(first: usize, last: usize, end: usize, element_size: u64) -> NameTagLayout {
    NameTagLayout { first, last, end, element_size }
}

#[test]
fn a_list_of_whole_elements_gives_its_count() {
    assert_eq!(layout(0x1000, 0x1000 + 3 * 144, 0x1000 + 8 * 144, 144).count(), Ok(3));
    assert_eq!(layout(0x1000, 0x1000, 0x1000 + 144, 144).count(), Ok(0));
}

#[test]
fn an_empty_vector_with_null_pointers_is_a_list_of_zero() {
    assert_eq!(layout(0, 0, 0, 144).count(), Ok(0));
}

#[test]
fn an_element_size_of_zero_is_a_fault() {
    assert_eq!(layout(0x1000, 0x1090, 0x1090, 0).count(), Err(LayoutFault::ZeroElementSize));
}

#[test]
fn pointers_out_of_order_are_a_fault() {
    assert_eq!(layout(0x2000, 0x1000, 0x3000, 144).count(), Err(LayoutFault::Unordered));
    assert_eq!(layout(0x1000, 0x3000, 0x2000, 144).count(), Err(LayoutFault::Unordered));
    assert_eq!(layout(0, 0x1000, 0x2000, 144).count(), Err(LayoutFault::Unordered));
}

#[test]
fn a_length_that_is_not_a_whole_number_of_elements_is_a_fault() {
    assert_eq!(layout(0x1000, 0x1000 + 145, 0x2000, 144).count(), Err(LayoutFault::Misaligned));
}

#[test]
fn read_takes_the_three_pointers_at_the_offset_both_builds_derived() {
    // `NameTagRenderObjectCollection.nameTagsFirst` was 0x10 on 26.51 and 26.60 before it was folded into the mirror
    let mut collection = [0_usize; 5];
    collection[2] = 0x1000;
    collection[3] = 0x1000 + 288;
    collection[4] = 0x2000;
    let base = (&raw const collection).cast::<c_void>();

    // SAFETY: the array is an aligned collection of the mirror size.
    let read = unsafe { NameTagLayout::read(base, 144) }.unwrap();

    assert_eq!(read, layout(0x1000, 0x1000 + 288, 0x2000, 144));
    assert_eq!(read.count(), Ok(2));
}
