use std::sync::atomic::{AtomicU32, Ordering};

/// Counts calls and reports the one call that reaches a target count.
pub struct TickCounter {
    target: u32,
    count: AtomicU32,
}

impl TickCounter {
    pub const fn new(target: u32) -> Self {
        Self { target, count: AtomicU32::new(0) }
    }

    /// Counts one call. True only for the call that reaches the target; the count stops there.
    pub fn tick(&self) -> bool {
        let previous = self
            .count
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |count| (count < self.target).then_some(count + 1));
        matches!(previous, Ok(count) if count + 1 == self.target)
    }

    pub fn target(&self) -> u32 {
        self.target
    }

    pub fn count(&self) -> u32 {
        self.count.load(Ordering::Relaxed)
    }
}
