use std::sync::atomic::{AtomicUsize, Ordering};

/// A reader of the live `DBStorage`: while one exists, `StorageDetours` does not let the storage's shutdown go on.
/// Made before the live address is loaded and dropped after the last read of the storage.
pub struct StorageReadGuard<'a> {
    readers: &'a AtomicUsize,
}

impl<'a> StorageReadGuard<'a> {
    /// Counts one reader in `readers`.
    pub fn new(readers: &'a AtomicUsize) -> Self {
        readers.fetch_add(1, Ordering::SeqCst);
        Self { readers }
    }
}

impl Drop for StorageReadGuard<'_> {
    fn drop(&mut self) {
        self.readers.fetch_sub(1, Ordering::SeqCst);
    }
}
