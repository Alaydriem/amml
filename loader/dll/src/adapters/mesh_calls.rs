use crate::{BaseActorRendererLayout, LoaderState, MeshContextLayout, MeshLayout, MeshLayoutError, TessellatorLayout};

/// The resolved game code and offsets of the world mesh path (`WorldCanvas::build` and `draw`): build a mesh in the
/// frame's tessellator, push a world matrix on the camera, set the actor constants and draw with a material.
///
/// `new` checks every mirror against its entry first: the `Tessellator` prefix (`Tessellator.mTessellating`), the
/// `mce::Mesh` size (`Mesh.size`), the `BaseActorRenderer` size (`BaseActorRenderer.size`), the `MeshContext` members
/// (`ScreenContext.camera`, `.constantBuffers` and `.currentShaderColor` relative to `ScreenContext.meshContext`) and
/// the `mce::Camera` layout (`Camera.mForward`, which puts the world matrix stack at 40h). A mismatch names its entry;
/// the loader then does not offer the area on this build.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MeshCalls {
    pub tessellator_begin: usize,
    pub tessellator_end: usize,
    pub tessellator_clear: usize,
    pub mesh_dtor: usize,
    pub mesh_is_valid: usize,
    pub mesh_render: usize,
    pub set_entity_constants: usize,
    pub matrix_stack_push: usize,
    pub matrix_stack_ref_dtor: usize,
    pub base_actor_renderer_ctor: usize,
    /// The offset of the `Tessellator&` in `ScreenContext`.
    pub screen_tessellator: usize,
    /// The offset of the `MeshContext` base in `ScreenContext`, the context the draw takes.
    pub screen_mesh_context: usize,
    /// The offset of `entityConstantBuffer` in `GlobalConstantBuffers`.
    pub entity_constant_buffer: usize,
    /// The offset of the `LevelRendererPlayer` pointer in `LevelRenderer`.
    pub level_renderer_player: usize,
    /// The offset of the `wireframe` `MaterialPtr` in `LevelRendererCamera`.
    pub wireframe_material: usize,
    /// The offset of the `RenderMaterial` pointer in `RenderMaterialInfo`.
    pub material_info_material: usize,
    /// The offsets in `RenderMaterial` of blend enable, blend source, blend destination and depth write.
    pub material_bytes: [usize; 4],
    /// The slot of `ClientInstance::getItemInHandRenderer` in the `ClientInstance` vtable.
    pub item_in_hand_renderer_slot: usize,
    /// The offset of the blend-block `MaterialPtr` in `ItemInHandRenderer`.
    pub blend_material: usize,
    /// The offset of the atlas `TexturePtr` in `LevelRenderer`.
    pub atlas_texture: usize,
}

impl MeshCalls {
    /// The offset of the world matrix stack in `mce::Camera`: after the view matrix stack (survey section 8.3).
    pub const CAMERA_WORLD_STACK: usize = 0x40;
    /// The offset of the dirty flag in a `MatrixStack` (survey sections 8.2 and 8.3).
    pub const MATRIX_STACK_DIRTY: usize = 0x38;
    /// The `Camera.mForward` value of the camera layout these offsets belong to.
    pub const CAMERA_FORWARD: u64 = 0x118;

    /// The calls of this build, from the resolved entries.
    pub fn new(state: &LoaderState) -> Result<Self, MeshLayoutError> {
        Self::from_lookup(&|name: &str| state.resolve(name).ok())
    }

    /// The calls from `lookup`, which gives the value of an entry (`None` when it does not resolve).
    pub fn from_lookup(lookup: &dyn Fn(&str) -> Option<u64>) -> Result<Self, MeshLayoutError> {
        let get = |entry: &'static str| lookup(entry).ok_or(MeshLayoutError::Missing { entry });
        let anchor = |entry: &'static str, expected: u64| {
            let found = get(entry)?;
            if found == expected { Ok(()) } else { Err(MeshLayoutError::Mismatch { entry, expected, found }) }
        };
        let mesh_context = get("ScreenContext.meshContext")?;
        anchor("Tessellator.mTessellating", TessellatorLayout::TESSELLATING)?;
        anchor("Mesh.size", MeshLayout::SIZE)?;
        anchor("BaseActorRenderer.size", BaseActorRendererLayout::SIZE)?;
        anchor("ScreenContext.camera", mesh_context + MeshContextLayout::CAMERA)?;
        anchor("ScreenContext.constantBuffers", mesh_context + MeshContextLayout::CONSTANT_BUFFERS)?;
        anchor("ScreenContext.currentShaderColor", mesh_context + MeshContextLayout::CURRENT_SHADER_COLOR)?;
        anchor("Camera.mForward", Self::CAMERA_FORWARD)?;
        let address = |entry: &'static str| {
            let value = get(entry)?;
            usize::try_from(value).ok().filter(|value| *value != 0).ok_or(MeshLayoutError::Mismatch {
                entry,
                expected: 1,
                found: value,
            })
        };
        let offset = |entry: &'static str| {
            let value = get(entry)?;
            usize::try_from(value).map_err(|_| MeshLayoutError::Mismatch { entry, expected: 0, found: value })
        };
        Ok(Self {
            tessellator_begin: address("Tessellator.begin")?,
            tessellator_end: address("Tessellator.end")?,
            tessellator_clear: address("Tessellator.clear")?,
            mesh_dtor: address("Mesh.dtor")?,
            mesh_is_valid: address("Mesh.isValid")?,
            mesh_render: address("Mesh.renderMeshTextureList")?,
            set_entity_constants: address("ActorShaderManager.setEntityConstants")?,
            matrix_stack_push: address("MatrixStack.push")?,
            matrix_stack_ref_dtor: address("MatrixStackRef.dtor")?,
            base_actor_renderer_ctor: address("BaseActorRenderer.ctor")?,
            screen_tessellator: offset("ScreenContext.tessellator")?,
            screen_mesh_context: offset("ScreenContext.meshContext")?,
            entity_constant_buffer: offset("GlobalConstantBuffers.entityConstantBuffer")?,
            level_renderer_player: offset("LevelRenderer.mLevelRendererPlayer")?,
            wireframe_material: offset("LevelRendererCamera.mWireframeMaterial")?,
            material_info_material: offset("RenderMaterialInfo.mMaterial")?,
            material_bytes: [
                offset("RenderMaterial.mEnableBlend")?,
                offset("RenderMaterial.mBlendSource")?,
                offset("RenderMaterial.mBlendDestination")?,
                offset("RenderMaterial.mDepthWriteMask")?,
            ],
            item_in_hand_renderer_slot: offset("ClientInstance.getItemInHandRenderer")?,
            blend_material: offset("ItemInHandRenderer.mMatBlendBlock")?,
            atlas_texture: offset("LevelRenderer.mAtlasTexture")?,
        })
    }
}
