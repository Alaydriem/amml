/// Why the mesh book refuses a call.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum MeshRefusal {
    #[error("the mesh is of an older generation, or was freed")]
    Stale,
    #[error("the mesh belongs to another mod")]
    NotOwner,
}
