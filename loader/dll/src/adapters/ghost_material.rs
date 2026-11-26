use crate::{ClientAdapter, MeshCalls, ObjectRead, TexturePtrLayout};

/// The material and the texture of a block mesh: the client's blend-block material (`ItemInHandRenderer.mMatBlendBlock`
/// of `ClientInstance::getItemInHandRenderer`) and the level renderer's terrain atlas (`LevelRenderer.mAtlasTexture`).
/// Both are read from the live objects at each draw; nothing is kept. Game thread, inside a `renderLevel` call.
pub struct GhostMaterial {
    calls: MeshCalls,
}

impl GhostMaterial {
    pub fn new(calls: MeshCalls) -> Self {
        Self { calls }
    }

    /// The address of the blend-block `MaterialPtr` of the item-in-hand renderer of `client`. `None` when the client
    /// gives no renderer or the material is empty: its first word is the `RenderMaterialInfo` the game reads, and the
    /// info holds the `RenderMaterial` at `RenderMaterialInfo.mMaterial`.
    ///
    /// # Safety
    /// `client` is the live primary `ClientInstance`, on the game thread.
    pub unsafe fn blend_material(&self, client: usize) -> Option<usize> {
        let renderer = ClientAdapter::call_pointer(client, Some(self.calls.item_in_hand_renderer_slot))
            .ok()
            .filter(|renderer| *renderer != 0)?;
        let material = renderer.checked_add(self.calls.blend_material)?;
        // SAFETY: the `MaterialPtr` is inline in the renderer (`ItemInHandRenderer.mMatBlendBlock`); its first word is
        // the info pointer.
        let info = unsafe { ObjectRead::pointer(material, 0) }?;
        // SAFETY: the info is a live `RenderMaterialInfo`; its material pointer is at the derived offset.
        unsafe { ObjectRead::pointer(info, self.calls.material_info_material) }?;
        Some(material)
    }

    /// A copy of the atlas `TexturePtr` of `renderer`, holding one use of each of its two control blocks, as the copy
    /// of the white texture does: the caller gives it to the game, which destroys it, or releases it. `None` while
    /// the renderer holds no atlas texture.
    ///
    /// # Safety
    /// `renderer` is the level renderer of the running `renderLevel` call, on its thread.
    pub unsafe fn atlas_copy(&self, renderer: usize) -> Option<TexturePtrLayout> {
        if renderer == 0 {
            return None;
        }
        let at = renderer.checked_add(self.calls.atlas_texture)?;
        // SAFETY: `LevelRenderer.mAtlasTexture` is an inline `TexturePtr` of the live renderer.
        let held = unsafe { &*std::ptr::with_exposed_provenance::<TexturePtrLayout>(at) };
        if held.data() == 0 {
            return None;
        }
        // SAFETY: each pointer of the held value is empty or holds a live use; the renderer keeps it for the call.
        Some(unsafe { TexturePtrLayout { texture: held.texture.clone_ref(), location: held.location.clone_ref() } })
    }
}
