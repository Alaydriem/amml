use std::ffi::c_void;
use std::mem::offset_of;

/// One node of the pool map of the game's entt registry (20h bytes): the index of the next node in the bucket chain, the
/// component type id (the key) and the pointer to the storage (the element of the value `shared_ptr`).
///
/// The 70.26 pool lookup (0x1450ce0) walks the chain with `mov rcx,[node]; cmp rcx,-1; shl rcx,5; cmp
/// [nodes+rcx+8],type; mov rax,[node+10h]`; 26.63 holds the inline copy of the same walk.
#[repr(C)]
pub struct EnttPoolNodeLayout {
    pub next: usize,         // 0x00 next node index, usize::MAX at the end of a chain
    pub type_id: u32,        // 0x08 key: entt type hash
    _pad: u32,               // 0x0C
    pub pool: *const c_void, // 0x10 shared_ptr element: the storage
    _control: *const c_void, // 0x18 shared_ptr control block
}

const _: () = assert!(offset_of!(EnttPoolNodeLayout, type_id) == 0x08);
const _: () = assert!(offset_of!(EnttPoolNodeLayout, pool) == 0x10);
const _: () = assert!(size_of::<EnttPoolNodeLayout>() == 0x20);
