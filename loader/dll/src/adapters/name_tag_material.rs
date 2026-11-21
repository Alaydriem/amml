use std::sync::OnceLock;

use crate::{BaseActorRendererLayout, MeshCalls};

/// `BaseActorRenderer::BaseActorRenderer()`: this in rcx, returned in rax.
type CtorFn = unsafe extern "C" fn(*mut u8) -> *mut u8;

/// The depth-tested name tag material (`mDepthTestedNameTagMat`) of one `BaseActorRenderer` the loader makes, which
/// draws the world meshes when Vibrant Visuals is off. The renderer is made on the game heap through the game's
/// constructor, once per process, on the first draw that needs it, and never freed (the material lives as long as
/// the process). A failed make is not tried again; it is logged once.
pub struct NameTagMaterial {
    calls: MeshCalls,
    // The renderer's address; 0 when the make failed.
    renderer: OnceLock<usize>,
}

impl NameTagMaterial {
    pub fn new(calls: MeshCalls) -> Self {
        Self { calls, renderer: OnceLock::new() }
    }

    /// The address of the material's `MaterialPtr`, making the renderer when it is not made yet. `None` when the make
    /// failed.
    ///
    /// # Safety
    /// `allocate` is the game heap's allocator; render thread, inside a `renderLevel` call.
    pub unsafe fn material(&self, allocate: extern "C" fn(usize) -> *mut u8) -> Option<usize> {
        let renderer = *self.renderer.get_or_init(|| {
            // SAFETY: passed on from the caller.
            let made = unsafe { self.make(allocate) };
            if made == 0 {
                curia::error!(
                    "the name tag renderer could not be made; world meshes are not drawn outside Vibrant Visuals"
                );
            }
            made
        });
        (renderer != 0).then(|| renderer + BaseActorRendererLayout::DEPTH_TESTED_NAME_TAG)
    }

    // Zeroed bytes of the mirror size from the game heap, then the game's constructor. 0 when the allocation fails.
    //
    // Safety: as `material`.
    unsafe fn make(&self, allocate: extern "C" fn(usize) -> *mut u8) -> usize {
        let size = size_of::<BaseActorRendererLayout>();
        let block = allocate(size);
        if block.is_null() {
            return 0;
        }
        // SAFETY: the allocation holds `size` bytes.
        unsafe { std::ptr::write_bytes(block, 0, size) };
        // SAFETY: the entry is the constructor with the signature of `CtorFn`; `BaseActorRenderer.size` was checked
        // against the mirror size.
        let ctor = unsafe { std::mem::transmute::<usize, CtorFn>(self.calls.base_actor_renderer_ctor) };
        // SAFETY: a zeroed block of the class size, aligned by the game heap.
        unsafe { ctor(block) };
        block.expose_provenance()
    }
}
