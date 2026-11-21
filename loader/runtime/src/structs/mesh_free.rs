use super::MeshKey;

/// One game mesh that the render thread must destroy: the name it had and the address the loader holds for it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MeshFree {
    pub key: MeshKey,
    pub handle: usize,
}
