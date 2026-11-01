use amml_game::OptionImplLayout;

use crate::support::GameMemory;

// writes `text` as a short MSVC std::string (SSO) at `at`
fn put_short(memory: &mut GameMemory, at: usize, text: &[u8]) {
    for (index, byte) in text.iter().enumerate() {
        memory.put::<u8>(at + index, *byte);
    }
    memory.put::<u64>(at + 0x10, text.len() as u64);
    memory.put::<u64>(at + 0x18, 15);
}

#[test]
fn reads_the_save_tag_the_id_and_the_option_type_at_the_game_offsets() {
    let mut memory = GameMemory::new(0x2A0);
    put_short(&mut memory, 0x188, b"gfx_vsync");
    memory.put::<i32>(0x1CC, 321);
    memory.put::<i32>(0x1D4, 5);

    // SAFETY: the buffer is 16-byte aligned, zeroed and as large as the mirror.
    let option = unsafe { &*memory.ptr().cast::<OptionImplLayout>() };

    assert_eq!(option.save_tag.to_str().unwrap(), "gfx_vsync");
    assert_eq!(option.id, 321);
    assert_eq!(option.option_type, 5);
}

#[test]
fn the_offsets_outside_the_impl_are_named() {
    assert_eq!(OptionImplLayout::OPTION_IMPL_OFFSET, 0x8);
    assert_eq!(OptionImplLayout::INT_OPTION_VALUE_OFFSET, 0x18);
}
