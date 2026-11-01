use std::sync::atomic::{AtomicBool, Ordering};

/// Raised by disable before the mod puts back the game state it changed, so no frame or tick still running can
/// publish or apply that state again afterwards; lowered by enable.
pub struct StopLatch {
    raised: AtomicBool,
}

impl StopLatch {
    /// A latch that is lowered.
    pub const fn new() -> Self {
        Self { raised: AtomicBool::new(false) }
    }

    /// Raises the latch.
    pub fn raise(&self) {
        self.raised.store(true, Ordering::SeqCst);
    }

    /// Lowers the latch.
    pub fn lower(&self) {
        self.raised.store(false, Ordering::SeqCst);
    }

    /// Whether the latch is raised.
    pub fn raised(&self) -> bool {
        self.raised.load(Ordering::SeqCst)
    }
}

impl Default for StopLatch {
    fn default() -> Self {
        Self::new()
    }
}
