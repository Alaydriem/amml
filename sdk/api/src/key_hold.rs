use amml_abi::InputReleaseHoldFn;

/// One injected key press. The key stays down in the game until the hold is released or dropped, or
/// the loader releases it (lease end, suspension, world leave, focus loss, disable); a hold the loader
/// released already is released again without effect.
#[must_use = "dropping the hold releases the key"]
pub struct KeyHold {
    release: InputReleaseHoldFn,
    id: u64,
}

impl KeyHold {
    pub(crate) fn new(release: InputReleaseHoldFn, id: u64) -> Self {
        Self { release, id }
    }

    /// Releases the key at the next frame; the same as dropping the hold.
    pub fn release(self) {
        drop(self);
    }
}

impl Drop for KeyHold {
    fn drop(&mut self) {
        // SAFETY: the loader accepts any hold id and gives `OK` for one already released.
        #[allow(unsafe_code)]
        let _ = unsafe { (self.release)(self.id) };
    }
}
