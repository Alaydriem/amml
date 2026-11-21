use amml_abi::InputReleaseHoldFn;

/// One injected mouse button press, under the rules of `KeyHold`.
#[must_use = "dropping the hold releases the button"]
pub struct ButtonHold {
    release: InputReleaseHoldFn,
    id: u64,
}

impl ButtonHold {
    pub(crate) fn new(release: InputReleaseHoldFn, id: u64) -> Self {
        Self { release, id }
    }

    /// Releases the button at the next frame; the same as dropping the hold.
    pub fn release(self) {
        drop(self);
    }
}

impl Drop for ButtonHold {
    fn drop(&mut self) {
        // SAFETY: the loader accepts any hold id and gives `OK` for one already released.
        #[allow(unsafe_code)]
        let _ = unsafe { (self.release)(self.id) };
    }
}
