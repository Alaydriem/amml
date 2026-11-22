use amml_abi::{AmmlMeshGroup, AmmlStr, RenderMesherGroupDiscardFn};

/// An open group of the block mesher (`BlockMesher::begin_group`): one buffer that several grids are appended to before
/// `BlockMesher::finish` makes it one mesh. It is an owned resource of the loader, bound to this mod and the world it
/// was opened in, and not `Copy` or `Clone`: `finish` takes it by value, so a finished group cannot be used again. A group
/// dropped without `finish` is discarded: its slot is free at once and the loader destroys its buffer on the render
/// thread (a loader whose render table is older than version 6 keeps it until the world is left). The loader drops a
/// group that is not finished when the world is left, when a render device reset or surface loss is seen and when the
/// mod is disabled; any use after that gives `Err(Stale)`. A mod holds at most 64 open groups.
#[derive(Debug)]
pub struct MeshGroup {
    raw: AmmlMeshGroup,
    discard: Option<RenderMesherGroupDiscardFn>,
    module: &'static str,
}

impl MeshGroup {
    /// The most open groups a mod holds.
    pub const MAX_OPEN: usize = 64;

    pub(crate) fn from_raw(
        raw: AmmlMeshGroup,
        discard: Option<RenderMesherGroupDiscardFn>,
        module: &'static str,
    ) -> Self {
        Self { raw, discard, module }
    }

    pub(crate) fn raw(&self) -> AmmlMeshGroup {
        self.raw
    }

    /// The raw group, for a call that ends it (`finish`): the drop no longer discards it.
    pub(crate) fn into_raw(self) -> AmmlMeshGroup {
        let raw = self.raw;
        std::mem::forget(self);
        raw
    }
}

impl PartialEq for MeshGroup {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

impl Eq for MeshGroup {}

impl Drop for MeshGroup {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        let Some(discard) = self.discard else { return };
        // A group the loader already ended gives `STALE`; there is nothing left to free then.
        // SAFETY: the table function is the loader's; the module id lives for the call.
        let _ = unsafe { discard(AmmlStr::from_str(self.module), self.raw) };
    }
}
