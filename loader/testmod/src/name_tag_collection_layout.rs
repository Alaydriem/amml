use amml_msvc::MsvcVec;

/// Layout mirror of the game's `NameTagRenderObjectCollection` (26.51 header): the `LinearAllocator` of its vector
/// (a pointer and a shared_ptr control block), then the MSVC vector pointers. The elements are read as bytes; their
/// size comes from `NameTagRenderObject.size`.
#[repr(C)]
pub struct NameTagCollectionLayout {
    _allocator: [u8; 0x10],     // 0x00 mNameTags: LinearAllocator
    pub name_tags: MsvcVec<u8>, // 0x10 mNameTags: first, last, end
}
