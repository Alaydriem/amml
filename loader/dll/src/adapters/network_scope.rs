use std::cell::RefCell;
use std::sync::atomic::{AtomicU64, Ordering};

use amml_msvc::{MsvcHeap, MsvcString};

use crate::NetworkEvent;

// The serial the next scope gets; 0 is kept for "no scope".
static NEXT_SERIAL: AtomicU64 = AtomicU64::new(1);

thread_local! {
    // The network events running on this thread, innermost last.
    static STACK: RefCell<Vec<NetworkEvent>> = const { RefCell::new(Vec::new()) };
}

/// A running network event on this thread: from `enter` until the value drops. A `network` table call answers only
/// inside the dispatch that opened the scope, on that thread; the skin handles of the scope are refused after it.
pub struct NetworkScope {
    // Whether `enter` pushed an entry that the drop pops.
    pushed: bool,
}

impl NetworkScope {
    /// Opens a scope for `event`.
    pub fn enter(event: NetworkEvent) -> Self {
        let pushed = STACK.try_with(|stack| stack.borrow_mut().push(event)).is_ok();
        Self { pushed }
    }

    /// The serial the next scope gets. Skin handles carry it.
    pub fn next_serial() -> u64 {
        NEXT_SERIAL.fetch_add(1, Ordering::Relaxed)
    }

    /// Runs `body` on the innermost running event of this thread; `None` when none runs.
    pub fn with<R>(body: impl FnOnce(&mut NetworkEvent) -> R) -> Option<R> {
        STACK.try_with(|stack| stack.borrow_mut().last_mut().map(body)).ok().flatten()
    }

    // Frees the geometry strings the event kept, on the game heap.
    fn release(event: &mut NetworkEvent, heap: &MsvcHeap) {
        for mut text in event.take_geometry() {
            let text: &mut MsvcString = &mut text;
            // SAFETY: the game built the string with its own allocator, which `heap` calls; it is not used again.
            unsafe { text.release(heap) };
        }
    }
}

impl Drop for NetworkScope {
    fn drop(&mut self) {
        if !self.pushed {
            return;
        }
        let popped = STACK.try_with(|stack| stack.borrow_mut().pop()).ok().flatten();
        if let Some(mut event) = popped {
            let heap = event.heap();
            Self::release(&mut event, &heap);
        }
    }
}
