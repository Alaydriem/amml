use std::ffi::c_void;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicPtr, Ordering};

use crate::StopLatch;

/// The `ClientInstance*` a hook saw last. Reads give nothing while the mod is stopped.
pub struct HeldClient {
    client: AtomicPtr<c_void>,
}

impl HeldClient {
    /// A holder with no client.
    pub const fn new() -> Self {
        Self { client: AtomicPtr::new(std::ptr::null_mut()) }
    }

    /// Holds `client` as the current pointer.
    pub fn set(&self, client: *mut c_void) {
        self.client.store(client, Ordering::SeqCst);
    }

    /// Drops the held pointer.
    pub fn clear(&self) {
        self.client.store(std::ptr::null_mut(), Ordering::SeqCst);
    }

    /// The held pointer; `None` while `latch` is raised or when none is held.
    pub fn get(&self, latch: &StopLatch) -> Option<NonNull<c_void>> {
        if latch.raised() {
            return None;
        }
        NonNull::new(self.client.load(Ordering::SeqCst))
    }
}

impl Default for HeldClient {
    fn default() -> Self {
        Self::new()
    }
}
