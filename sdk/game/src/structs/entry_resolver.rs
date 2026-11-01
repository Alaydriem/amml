use amml_sdk::{Api, SdkError};

/// Something that gives the value of an entry id.
pub trait EntryResolver {
    /// The value of the entry `id`: an absolute address, a slot index or a byte offset, by the entry kind.
    fn resolve(&self, id: &str) -> Result<u64, SdkError>;
}

impl EntryResolver for Api {
    fn resolve(&self, id: &str) -> Result<u64, SdkError> {
        Api::resolve(self, id)
    }
}
