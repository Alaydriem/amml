use std::ffi::c_void;

use amml_game::ActorRenderQueueLayout;

use crate::support::GameMemory;

#[test]
fn reads_the_key_and_value_vectors_at_the_game_offsets() {
    let mut memory = GameMemory::new(0x30);
    // keys begin/end/cap +0/+8/+0x10, values begin/end/cap +0x18/+0x20/+0x28
    memory.put::<usize>(0x00, 0x1000);
    memory.put::<usize>(0x08, 0x1010);
    memory.put::<usize>(0x10, 0x1020);
    memory.put::<usize>(0x18, 0x2000);
    memory.put::<usize>(0x20, 0x2002);
    memory.put::<usize>(0x28, 0x2004);

    // SAFETY: the buffer is 16-byte aligned, zeroed and as large as the mirror.
    let queue = unsafe { &*memory.ptr().cast::<ActorRenderQueueLayout>() };

    assert_eq!(queue.keys_begin, 0x1000 as *mut *mut c_void);
    assert_eq!(queue.keys_end, 0x1010 as *mut *mut c_void);
    assert_eq!(queue.keys_cap, 0x1020 as *mut *mut c_void);
    assert_eq!(queue.values_begin, 0x2000 as *mut u8);
    assert_eq!(queue.values_end, 0x2002 as *mut u8);
    assert_eq!(queue.values_cap, 0x2004 as *mut u8);
}
