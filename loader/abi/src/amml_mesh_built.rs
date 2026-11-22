use crate::AmmlMeshId;

/// What one block mesher build gave: the mesh (id 0 when nothing was drawn), its quads, and how many cells
/// `skip_matching_world` left out.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AmmlMeshBuilt {
    pub mesh: AmmlMeshId,
    pub quads: u32,
    pub skipped_count: u32,
}

const _: () = assert!(core::mem::size_of::<AmmlMeshBuilt>() == 24);
