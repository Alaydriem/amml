/// A mesh group of the block mesher: one buffer that several grids are appended to before it becomes one mesh. A group of
/// an older generation is `STALE`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct AmmlMeshGroup {
    pub id: u32,
    pub generation: u32,
}

const _: () = assert!(core::mem::size_of::<AmmlMeshGroup>() == 8);
