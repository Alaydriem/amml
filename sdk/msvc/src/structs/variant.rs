/// Layout mirror of the MSVC `std::variant`: the storage of the largest alternative, then the index (one byte for
/// fewer than 255 alternatives).
#[repr(C)]
pub struct MsvcVariant<S: Copy> {
    storage: S,
    index: u8,
}

impl<S: Copy> Clone for MsvcVariant<S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S: Copy> Copy for MsvcVariant<S> {}

impl<S: Copy> MsvcVariant<S> {
    /// A variant that holds the alternative `index`, whose bytes are in `storage`.
    pub fn new(index: u8, storage: S) -> Self {
        MsvcVariant { storage, index }
    }

    pub fn index(&self) -> u8 {
        self.index
    }

    pub fn storage(&self) -> &S {
        &self.storage
    }
}
