/// A texture of the loader. `id` is never given twice in a process. `generation` counts the times the loader freed
/// every texture (a world leave): an id of an older generation is `STALE`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct AmmlTextureId {
    pub id: u64,
    pub generation: u32,
    pub _reserved: u32,
}

const _: () = assert!(core::mem::size_of::<AmmlTextureId>() == 16);
