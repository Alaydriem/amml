use std::ffi::c_void;

use amml_game::ReadOnlyBinaryStreamLayout;

use crate::support::GameMemory;

#[test]
fn reads_the_view_read_pointer_and_overflow_at_the_stream_offsets() {
    let bytes = *b"request";
    let mut memory = GameMemory::new(0x48);
    // ReadOnlyBinaryStream: vptr 0x00, mOwnedBuffer 0x08, mView 0x28 (data, size), mReadPointer 0x38,
    // mHasOverflowed 0x40
    memory.put::<usize>(0x00, 0x7FF6_0000_2000);
    memory.put::<usize>(0x28, bytes.as_ptr() as usize);
    memory.put::<usize>(0x30, bytes.len());
    memory.put::<u64>(0x38, 5);
    memory.put::<u8>(0x40, 1);

    // SAFETY: the buffer is 16-byte aligned, zeroed and as large as the mirror.
    let stream = unsafe { &*memory.ptr().cast::<ReadOnlyBinaryStreamLayout>() };

    assert_eq!(stream.vtable, 0x7FF6_0000_2000 as *const c_void);
    assert_eq!(stream.view, bytes.as_ptr());
    assert_eq!(stream.view_len, 7);
    assert_eq!(stream.read_pointer, 5);
    assert!(stream.has_overflowed);
}

#[test]
fn reads_the_owned_buffer_as_an_msvc_string() {
    let mut memory = GameMemory::new(0x48);
    // MSVC std::string at 0x08: 16-byte inline buffer, size 0x18, capacity 0x20 (15 for an inline text)
    memory.put::<[u8; 5]>(0x08, *b"bytes");
    memory.put::<usize>(0x18, 5);
    memory.put::<usize>(0x20, 15);

    // SAFETY: as above.
    let stream = unsafe { &*memory.ptr().cast::<ReadOnlyBinaryStreamLayout>() };

    assert_eq!(stream.owned_buffer.as_bytes(), b"bytes");
}
