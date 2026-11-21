use amml_abi::AmmlMeshId;

/// A mesh of the loader. `id` is never given twice in a process. `generation` counts the times the loader freed every
/// mesh (a world leave, a render device reset, a switch of the graphics mode): a `MeshId` of an older generation gives
/// `Stale`. Free it with `WorldCanvas::free`; the loader also frees it when its mod is disabled.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MeshId {
    pub id: u64,
    pub generation: u32,
}

impl From<AmmlMeshId> for MeshId {
    fn from(mesh: AmmlMeshId) -> Self {
        Self { id: mesh.id, generation: mesh.generation }
    }
}

impl From<MeshId> for AmmlMeshId {
    fn from(mesh: MeshId) -> Self {
        Self { id: mesh.id, generation: mesh.generation, _reserved: 0 }
    }
}
