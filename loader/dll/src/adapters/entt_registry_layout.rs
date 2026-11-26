use std::mem::offset_of;

use amml_msvc::MsvcVec;

use crate::EnttPoolNodeLayout;

/// Layout mirror of the head of the game's `entt::basic_registry`, up to its pool map (a `dense_map` of type id to
/// `shared_ptr` of the storage): the bucket vector of node indexes (a bucket is `type id & (count - 1)`), the hasher,
/// then the node vector. The same on 26.63 and 70.26 (the 70.26 pool lookup reads the bucket vector at 48h and 50h and
/// the node vector at 68h and 70h).
#[repr(C)]
pub struct EnttRegistryLayout {
    _context: [u64; 9],                     // 0x00 the context variables
    pub buckets: MsvcVec<usize>,            // 0x48 the bucket heads
    _hasher: usize,                         // 0x60
    pub nodes: MsvcVec<EnttPoolNodeLayout>, // 0x68 the nodes
}

const _: () = assert!(offset_of!(EnttRegistryLayout, buckets) == 0x48);
const _: () = assert!(offset_of!(EnttRegistryLayout, nodes) == 0x68);
