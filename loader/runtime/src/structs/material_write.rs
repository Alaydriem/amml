use super::MaterialBytes;

/// The bytes the render thread writes into the `RenderMaterial` at `address`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaterialWrite {
    pub address: usize,
    pub bytes: MaterialBytes,
}
