use crate::{MeshBuilder, MeshCalls, MeshDrawAdapter, NameTagMaterial, WorldMaterial};

/// The parts of the world mesh path, made only when every mirror fits its entry: the builder, the draw, the wireframe
/// material patch and the name tag material.
pub struct MeshParts {
    pub builder: MeshBuilder,
    pub drawer: MeshDrawAdapter,
    pub material: WorldMaterial,
    pub name_tag: NameTagMaterial,
}

impl MeshParts {
    pub fn new(calls: MeshCalls) -> Self {
        Self {
            builder: MeshBuilder::new(calls),
            drawer: MeshDrawAdapter::new(calls),
            material: WorldMaterial::new(calls),
            name_tag: NameTagMaterial::new(calls),
        }
    }
}
