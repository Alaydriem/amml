use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

// One flag the app sets and every download reads. Clones share the same flag.
#[derive(Debug, Clone, Default)]
pub struct CancelFlag {
    cancelled: Arc<AtomicBool>,
}

impl CancelFlag {
    pub fn new() -> CancelFlag {
        CancelFlag { cancelled: Arc::new(AtomicBool::new(false)) }
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    pub fn reset(&self) {
        self.cancelled.store(false, Ordering::SeqCst);
    }
}
