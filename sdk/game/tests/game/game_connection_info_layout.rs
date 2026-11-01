use amml_game::GameConnectionInfoLayout;

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
fn the_head_reads_the_type_host_url_and_port_at_the_header_offsets() {
    let mut memory = GameMemory::new(0x90);
    memory.put::<u16>(0x00, 2);
    put_short(&mut memory, 0x08, b"203.0.113.7");
    put_short(&mut memory, 0x28, b"mc.example.net");
    memory.put::<u32>(0x8C, 19132);

    // SAFETY: the buffer is 16-byte aligned, zeroed and as large as the mirror.
    let info = unsafe { &*memory.ptr().cast::<GameConnectionInfoLayout>() };

    assert_eq!(info.connection_type, 2);
    assert_eq!(info.host.to_str().unwrap(), "203.0.113.7");
    assert_eq!(info.url.to_str().unwrap(), "mc.example.net");
    assert_eq!(info.port, 19132);
}

#[test]
fn an_empty_host_reads_as_an_empty_string() {
    let mut memory = GameMemory::new(0x90);
    memory.put::<u64>(0x08 + 0x18, 15);

    // SAFETY: as above.
    let info = unsafe { &*memory.ptr().cast::<GameConnectionInfoLayout>() };

    assert_eq!(info.host.to_str().unwrap(), "");
}
