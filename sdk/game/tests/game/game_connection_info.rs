use amml_game::{GameConnectionInfo, GameConnectionInfoLayout};

use crate::support::GameMemory;

// writes `text` as an MSVC std::string at `at`: inline up to 15 bytes, else through a heap block kept in `heap`
fn put_string(memory: &mut GameMemory, heap: &mut Vec<Vec<u8>>, at: usize, text: &[u8]) {
    memory.put::<u64>(at + 0x10, text.len() as u64);
    if text.len() <= 15 {
        for (index, byte) in text.iter().enumerate() {
            memory.put::<u8>(at + index, *byte);
        }
        memory.put::<u64>(at + 0x18, 15);
    } else {
        let mut block = text.to_vec();
        block.push(0);
        memory.put::<u64>(at, block.as_ptr() as u64);
        memory.put::<u64>(at + 0x18, text.len() as u64);
        heap.push(block);
    }
}

// a host on the heap with `capacity` set, and different bytes in the inline buffer
fn read_stale_inline(host: &[u8], capacity: u64) -> String {
    let mut memory = GameMemory::new(0x90);
    let mut block = host.to_vec();
    block.push(0);
    memory.put::<u64>(0x08, block.as_ptr() as u64);
    memory.put::<u64>(0x08 + 8, 0x5858_5858_5858_5858);
    memory.put::<u64>(0x08 + 0x10, host.len() as u64);
    memory.put::<u64>(0x08 + 0x18, capacity);
    // SAFETY: the buffer is 16-byte aligned, zeroed and as large as the mirror; `block` outlives the read.
    let layout = unsafe { &*memory.ptr().cast::<GameConnectionInfoLayout>() };
    GameConnectionInfo::new(layout).host()
}

fn read(host: &[u8], port: u32) -> (String, u16) {
    let mut memory = GameMemory::new(0x90);
    let mut heap = Vec::new();
    put_string(&mut memory, &mut heap, 0x08, host);
    memory.put::<u32>(0x8C, port);
    // SAFETY: the buffer is 16-byte aligned, zeroed and as large as the mirror; the heap blocks live in `heap`.
    let layout = unsafe { &*memory.ptr().cast::<GameConnectionInfoLayout>() };
    let info = GameConnectionInfo::new(layout);
    (info.host(), info.port())
}

#[test]
fn an_empty_host_gives_an_empty_string_and_the_port() {
    assert_eq!(read(b"", 19132), (String::new(), 19132));
}

#[test]
fn a_host_of_fifteen_bytes_stays_inline() {
    let host = b"123.123.123.123";
    assert_eq!(host.len(), 15);
    assert_eq!(read(host, 25565), ("123.123.123.123".to_string(), 25565));
}

#[test]
fn a_host_of_forty_bytes_is_read_from_the_heap_block() {
    let host = b"0123456789012345678901234567890123456789";
    assert_eq!(host.len(), 40);
    assert_eq!(read(host, 1), (String::from_utf8(host.to_vec()).unwrap(), 1));
}

#[test]
fn a_port_that_does_not_fit_sixteen_bits_gives_zero() {
    assert_eq!(read(b"a", 70000).1, 0);
    assert_eq!(read(b"a", u32::MAX).1, 0);
}

#[test]
fn a_host_of_sixteen_bytes_is_the_first_size_on_the_heap() {
    let host = b"0123456789abcdef";
    assert_eq!(host.len(), 16);
    assert_eq!(read(host, 2).0, "0123456789abcdef");
}

#[test]
fn a_short_host_with_a_heap_capacity_is_read_from_the_heap() {
    // the game made a long host short again: the size is 5, the capacity is 31, the bytes are in the heap block
    assert_eq!(read_stale_inline(b"short", 31), "short");
}
