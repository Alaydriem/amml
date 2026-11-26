use crate::MeshId;

/// What `BlockMesher::build` made: the mesh (draw it with `WorldCanvas::draw`, free it with `WorldCanvas::free`), its
/// quads, and the cell indices `MesherOptions::skip_matching_world` left out, rising. With `quads` 0 nothing was drawn:
/// `mesh` is then an id of no mesh, which every draw refuses with `Stale`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MeshBuilt {
    pub mesh: MeshId,
    pub quads: u32,
    pub skipped: Vec<u32>,
}
