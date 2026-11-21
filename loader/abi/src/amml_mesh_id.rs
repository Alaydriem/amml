/// A mesh of the loader. `id` is never given twice in a process. `generation` counts the times the loader freed every
/// mesh (a world leave, a render device reset, a graphics mode switch): an id of an older generation is `STALE`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct AmmlMeshId {
    pub id: u64,
    pub generation: u32,
    pub _reserved: u32,
}

const _: () = assert!(core::mem::size_of::<AmmlMeshId>() == 16);
