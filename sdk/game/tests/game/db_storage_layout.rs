use amml_game::DbStorageLayout;

use crate::support::GameMemory;

#[test]
fn the_full_path_reads_at_the_header_offset() {
    let mut memory = GameMemory::new(0xB8);
    let path = b"worlds/My World";
    for (index, byte) in path.iter().enumerate() {
        memory.put::<u8>(0x98 + index, *byte);
    }
    memory.put::<u64>(0x98 + 0x10, path.len() as u64);
    memory.put::<u64>(0x98 + 0x18, 15);

    // SAFETY: the buffer is 16-byte aligned, zeroed and as large as the mirror.
    let storage = unsafe { &*memory.ptr().cast::<DbStorageLayout>() };

    assert_eq!(storage.full_path.to_str().unwrap(), "worlds/My World");
}

#[test]
fn an_empty_path_reads_as_an_empty_string() {
    let mut memory = GameMemory::new(0xB8);
    memory.put::<u64>(0x98 + 0x18, 15);

    // SAFETY: as above.
    let storage = unsafe { &*memory.ptr().cast::<DbStorageLayout>() };

    assert_eq!(storage.full_path.to_str().unwrap(), "");
}
