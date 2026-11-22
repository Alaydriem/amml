use crate::{GhostMaterial, MeshBuilder, MeshCalls, MeshDrawAdapter, NameTagMaterial, WorldMaterial};

/// The parts of the world mesh path, made only when every mirror fits its entry: the builder, the draw, the wireframe
/// material patch, the name tag material and the ghost material.
pub struct MeshParts {
    pub builder: MeshBuilder,
    pub drawer: MeshDrawAdapter,
    pub material: WorldMaterial,
    pub name_tag: NameTagMaterial,
    pub ghost: GhostMaterial,
}

impl MeshParts {
    pub fn new(calls: MeshCalls) -> Self {
        Self {
            builder: MeshBuilder::new(calls),
            drawer: MeshDrawAdapter::new(calls),
            material: WorldMaterial::new(calls),
            name_tag: NameTagMaterial::new(calls),
            ghost: GhostMaterial::new(calls),
        }
    }
}
