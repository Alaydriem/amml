use std::mem::offset_of;

use amml_msvc::MsvcVec;

/// Layout mirror of the head of the game's entt storage of one component, up to its payload pages: the sparse set (its
/// vtable, the sparse page vector, the packed vector and its bookkeeping), then the payload page vector. The same on
/// 26.63 and 70.26 (`setupCamera` reads the sparse pages at 08h and 10h and the payload pages at 50h on both).
#[repr(C)]
pub struct EnttStorageLayout {
    _vtable: usize,                  // 0x00
    pub sparse: MsvcVec<*const u32>, // 0x08 the sparse pages
    _packed: [u64; 6],               // 0x20 the packed vector, mode and head
    pub payload: MsvcVec<*const u8>, // 0x50 the payload pages
}

const _: () = assert!(offset_of!(EnttStorageLayout, sparse) == 0x08);
const _: () = assert!(offset_of!(EnttStorageLayout, payload) == 0x50);
