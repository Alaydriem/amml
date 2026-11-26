use crate::NonOwnerPointerLayout;

/// The `Bedrock::NonOwnerPointer<CameraRegistry>` that `getCameraRegistry` handed out. It holds one use of the control
/// block, which `Drop` gives back on every path, a panic included.
pub struct CameraRegistryHold {
    pointer: NonOwnerPointerLayout,
}

impl CameraRegistryHold {
    /// Takes over a pointer the game just handed out.
    ///
    /// # Safety
    /// `pointer` was written by the game's `getCameraRegistry` and holds its own use of the control block (or none when
    /// it is empty); nothing else releases it.
    #[allow(unsafe_code)]
    pub unsafe fn new(pointer: NonOwnerPointerLayout) -> Self {
        Self { pointer }
    }

    /// The registry the pointer names; `None` when the pointer is empty or its valid flag is not 1.
    pub fn registry(&self) -> Option<usize> {
        (self.pointer.control.get().copied() == Some(1) && !self.pointer.pointer.is_null())
            .then(|| self.pointer.pointer.expose_provenance())
    }
}

impl Drop for CameraRegistryHold {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        // SAFETY: `new` was given one use of the control block, and this is the only release.
        unsafe { self.pointer.control.release() };
    }
}
