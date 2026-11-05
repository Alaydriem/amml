use std::sync::{Arc, Mutex, MutexGuard};

// One lock for the registry cache folder. Every `RegistryCache` on the folder shares a clone,
// so a body and its etag are never read or replaced half way by another thread.
#[derive(Clone, Default)]
pub struct CacheLock {
    inner: Arc<Mutex<()>>,
}

impl CacheLock {
    pub fn new() -> CacheLock {
        CacheLock { inner: Arc::new(Mutex::new(())) }
    }

    // A lock that a panic poisoned still works: the cache files are safe to read again.
    pub fn hold(&self) -> MutexGuard<'_, ()> {
        self.inner.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
