use std::ffi::c_void;
use std::sync::atomic::{AtomicPtr, Ordering};

use windows::Win32::Graphics::Direct3D12::ID3D12CommandQueue;
use windows::core::Interface;

/// The game's first D3D12 DIRECT queue, seen in `ExecuteCommandLists`, with one reference held. Any thread may offer
/// a queue; only the render thread, inside a Present or ResizeBuffers detour, takes it away. Nothing releases it at
/// shutdown: the render thread may still use it, and a reference left at process exit is harmless.
pub struct QueueSlot {
    queue: AtomicPtr<c_void>,
}

impl Default for QueueSlot {
    fn default() -> Self {
        Self::new()
    }
}

impl QueueSlot {
    pub const fn new() -> Self {
        Self { queue: AtomicPtr::new(std::ptr::null_mut()) }
    }

    pub fn is_empty(&self) -> bool {
        self.queue.load(Ordering::SeqCst).is_null()
    }

    /// The kept queue as a plain value, for identity only; 0 when none.
    pub fn identity(&self) -> usize {
        self.queue.load(Ordering::SeqCst).expose_provenance()
    }

    /// Keeps `queue` when the slot is empty.
    pub fn offer(&self, queue: &ID3D12CommandQueue) {
        if !self.is_empty() {
            return;
        }
        let raw = queue.clone().into_raw();
        if self.queue.compare_exchange(std::ptr::null_mut(), raw, Ordering::SeqCst, Ordering::SeqCst).is_err() {
            // SAFETY: `raw` holds the reference taken above, which was not stored.
            drop(unsafe { ID3D12CommandQueue::from_raw(raw) });
        }
    }

    /// A new reference to the kept queue. Render thread only.
    pub fn get(&self) -> Option<ID3D12CommandQueue> {
        let raw = self.queue.load(Ordering::SeqCst);
        // SAFETY: a stored pointer holds the slot's own reference, and only the render thread (the caller) releases
        // it.
        unsafe { ID3D12CommandQueue::from_raw_borrowed(&raw) }.cloned()
    }

    /// Drops the kept queue, if any, so the next submit can be kept. Render thread only.
    pub fn clear(&self) {
        let raw = self.queue.swap(std::ptr::null_mut(), Ordering::SeqCst);
        if !raw.is_null() {
            // SAFETY: the slot held one reference to `raw`, now taken out of it.
            drop(unsafe { ID3D12CommandQueue::from_raw(raw) });
        }
    }
}
