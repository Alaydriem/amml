use std::sync::atomic::{AtomicU32, Ordering};

/// Lets the first call and then every `every`-th call through.
pub struct Throttle {
    every: u32,
    count: AtomicU32,
}

impl Throttle {
    /// A throttle that passes one call in `every`; an `every` of 0 passes every call.
    pub const fn new(every: u32) -> Self {
        Self { every, count: AtomicU32::new(0) }
    }

    /// Counts one call. True for the first call and for each `every`-th call after it.
    pub fn hit(&self) -> bool {
        let previous = self.count.fetch_add(1, Ordering::Relaxed);
        self.every == 0 || previous.is_multiple_of(self.every)
    }
}
