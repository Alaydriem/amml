use amml_game::{OptionImplLayout, OptionReader};

use crate::support::GameMemory;

// a fake Option: the Impl pointer at +8 and the int value at +0x18
fn option(implementation: &mut GameMemory, value: i32) -> GameMemory {
    let mut option = GameMemory::new(0x20);
    option.put::<u64>(OptionImplLayout::OPTION_IMPL_OFFSET, implementation.ptr() as u64);
    option.put::<i32>(OptionImplLayout::INT_OPTION_VALUE_OFFSET, value);
    option
}

#[test]
fn reads_the_id_the_type_the_save_tag_and_the_int_value() {
    let mut implementation = GameMemory::new(0x2A0);
    // the save tag is 17 bytes, so it lives in a heap block
    let block = b"gfx_max_framerate\0".to_vec();
    implementation.put::<u64>(0x188, block.as_ptr() as u64);
    implementation.put::<u64>(0x188 + 0x10, 17);
    implementation.put::<u64>(0x188 + 0x18, 31);
    implementation.put::<i32>(0x1CC, 321);
    implementation.put::<i32>(0x1D4, 5);
    let mut option = option(&mut implementation, 144);

    // SAFETY: a live fake option whose Impl pointer is set.
    let reader = unsafe { OptionReader::from_raw(option.ptr()) }.unwrap();

    assert_eq!(reader.id(), 321);
    assert_eq!(reader.option_type(), 5);
    assert_eq!(reader.save_tag(), "gfx_max_framerate");
    assert_eq!(reader.int_value(), Some(144));
}

#[test]
fn a_short_save_tag_is_read_inline() {
    let mut implementation = GameMemory::new(0x2A0);
    for (index, byte) in b"vsync".iter().enumerate() {
        implementation.put::<u8>(0x188 + index, *byte);
    }
    implementation.put::<u64>(0x188 + 0x10, 5);
    implementation.put::<u64>(0x188 + 0x18, 15);
    let mut option = option(&mut implementation, 0);

    // SAFETY: as above.
    let reader = unsafe { OptionReader::from_raw(option.ptr()) }.unwrap();

    assert_eq!(reader.save_tag(), "vsync");
}

#[test]
fn a_null_option_or_a_null_impl_gives_none() {
    // SAFETY: null is allowed.
    assert!(unsafe { OptionReader::from_raw(std::ptr::null_mut()) }.is_none());

    let mut empty = GameMemory::new(0x20);
    // SAFETY: a readable option whose Impl pointer is null.
    assert!(unsafe { OptionReader::from_raw(empty.ptr()) }.is_none());
}

#[test]
fn int_value_is_none_unless_the_option_type_is_int() {
    for (option_type, expected) in [(5, Some(144)), (6, None), (0, None)] {
        let mut implementation = GameMemory::new(0x2A0);
        implementation.put::<i32>(0x1D4, option_type);
        let mut option = option(&mut implementation, 144);

        // SAFETY: as above.
        let reader = unsafe { OptionReader::from_raw(option.ptr()) }.unwrap();

        assert_eq!(reader.int_value(), expected, "type {option_type}");
    }
}
