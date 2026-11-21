use amml_abi::ProviderReady;

/// Typed reads of a `ProviderReady`.
pub trait ProviderReadyExt {
    /// The name of the provider table, for the listener call it is read in; `None` when the loader's view
    /// is not text.
    fn name(&self) -> Option<&str>;
}

impl ProviderReadyExt for ProviderReady {
    fn name(&self) -> Option<&str> {
        // SAFETY: the loader keeps `name` valid for the listener call, and the borrow of `self` (the event
        // the listener got) does not outlive that call.
        #[allow(unsafe_code)]
        unsafe {
            self.name.as_str().ok()
        }
    }
}
