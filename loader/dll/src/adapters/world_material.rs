use amml_runtime::{MaterialBytes, MaterialChoice, MaterialPatch, MaterialWrite};

use crate::{MeshCalls, ObjectRead};

/// The level renderer's `wireframe` material and its patch (`MaterialPatch`): reads the material through
/// `LevelRenderer.mLevelRendererPlayer` -> the inline `MaterialPtr` (`LevelRendererCamera.mWireframeMaterial`) ->
/// `RenderMaterialInfo.mMaterial` -> the `RenderMaterial`, and writes the patch and the restore into its four bytes.
/// Every write runs on the render thread inside a `renderLevel` call, into the material the running renderer has.
pub struct WorldMaterial {
    calls: MeshCalls,
    patch: MaterialPatch,
}

impl WorldMaterial {
    pub fn new(calls: MeshCalls) -> Self {
        Self { calls, patch: MaterialPatch::new() }
    }

    /// The `MaterialPtr` and the `RenderMaterial` of the wireframe material of `renderer`; `None` when a hop is null.
    ///
    /// # Safety
    /// `renderer` is zero or the level renderer of the running `renderLevel` call, on its thread.
    pub unsafe fn wireframe(&self, renderer: usize) -> Option<(usize, usize)> {
        // SAFETY: the caller guarantees a live renderer; its player pointer is at the derived offset.
        let player = unsafe { ObjectRead::pointer(renderer, self.calls.level_renderer_player) }?;
        let material_ptr = player.checked_add(self.calls.wireframe_material)?;
        // SAFETY: the `MaterialPtr` is inline in the player's camera base; its first word is the info pointer.
        let info = unsafe { ObjectRead::pointer(material_ptr, 0) }?;
        // SAFETY: the info is a live `RenderMaterialInfo`; its material pointer is at the derived offset.
        let material = unsafe { ObjectRead::pointer(info, self.calls.material_info_material) }?;
        Some((material_ptr, material))
    }

    /// Before one draw of a mesh of `owner`: writes the restore that is due or that the mode needs, then, in Vibrant
    /// Visuals, the patch. Gives the material to draw with: the wireframe `MaterialPtr`, or `None` for the name tag
    /// material.
    ///
    /// # Safety
    /// As `wireframe`.
    pub unsafe fn prepare(&self, owner: usize, vibrant: bool, renderer: usize) -> (MaterialChoice, Option<usize>) {
        // SAFETY: passed on from the caller.
        let wireframe = unsafe { self.wireframe(renderer) };
        // SAFETY: the material was read from the live renderer in this call.
        let current = wireframe.map(|(_, material)| (material, unsafe { self.read(material) }));
        let plan = self.patch.prepare(owner, vibrant, current);
        for write in &plan.writes {
            // SAFETY: each write names the material read from the live renderer in this call.
            unsafe { self.write(write) };
        }
        let material_ptr = wireframe.map(|(material_ptr, _)| material_ptr);
        (plan.choice, material_ptr.filter(|_| plan.choice == MaterialChoice::Wireframe))
    }

    /// At a `renderLevel` call: writes the restore when one is due and the material is the kept one.
    ///
    /// # Safety
    /// As `wireframe`.
    pub unsafe fn restore_due_now(&self, renderer: usize) {
        if !self.patch.restore_due() {
            return;
        }
        // SAFETY: passed on from the caller.
        let current = unsafe { self.wireframe(renderer) }.map(|(_, material)| material);
        if let Some(write) = self.patch.take_restore(current) {
            // SAFETY: the write names the material read from the live renderer in this call.
            unsafe { self.write(&write) };
            curia::info!("world material restored", { reason: "restore" });
        }
    }

    /// Asks for the restore at the next render call.
    pub fn request_restore(&self) {
        self.patch.request_restore();
    }

    /// The mod at `owner` is disabled.
    pub fn disable_owner(&self, owner: usize) {
        self.patch.disable_owner(owner);
    }

    /// Whether the material is patched or waits for its restore.
    pub fn is_applied(&self) -> bool {
        self.patch.is_applied()
    }

    // Safety: `material` is a live `RenderMaterial` read in this render call.
    unsafe fn read(&self, material: usize) -> MaterialBytes {
        // SAFETY: the offsets are the material's derived byte members.
        let byte = |at: usize| unsafe { ObjectRead::value::<u8>(material, at) }.unwrap_or(0);
        let [blend, source, destination, depth] = self.calls.material_bytes;
        MaterialBytes {
            enable_blend: byte(blend),
            blend_source: byte(source),
            blend_destination: byte(destination),
            depth_write: byte(depth),
        }
    }

    // Safety: `write.address` is a live `RenderMaterial` read in this render call.
    unsafe fn write(&self, write: &MaterialWrite) {
        let bytes = write.bytes;
        let values = [bytes.enable_blend, bytes.blend_source, bytes.blend_destination, bytes.depth_write];
        for (at, value) in self.calls.material_bytes.into_iter().zip(values) {
            // SAFETY: the material is live and the offset is one of its derived byte members.
            unsafe { std::ptr::with_exposed_provenance_mut::<u8>(write.address + at).write(value) };
        }
    }
}
