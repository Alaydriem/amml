use std::sync::atomic::{AtomicU64, Ordering};

/// The frame times of one measuring window of the overlay checks, summed on the render thread by the
/// `FramePresented` listener: the time between two presents, and the time the game's own `Present`
/// call took (`cpu_us`).
pub struct FrameCost {
    frames: AtomicU64,
    interval_us: AtomicU64,
    present_us: AtomicU64,
    // The longest time between two presents, for a stall at the start of the window.
    worst_us: AtomicU64,
}

impl Default for FrameCost {
    fn default() -> Self {
        Self::new()
    }
}

impl FrameCost {
    pub const fn new() -> Self {
        Self {
            frames: AtomicU64::new(0),
            interval_us: AtomicU64::new(0),
            present_us: AtomicU64::new(0),
            worst_us: AtomicU64::new(0),
        }
    }

    /// Counts one present that came `interval_us` after the one before and took `present_us`.
    pub fn add(&self, interval_us: u64, present_us: u64) {
        self.interval_us.fetch_add(interval_us, Ordering::AcqRel);
        self.present_us.fetch_add(present_us, Ordering::AcqRel);
        self.worst_us.fetch_max(interval_us, Ordering::AcqRel);
        self.frames.fetch_add(1, Ordering::AcqRel);
    }

    pub fn frames(&self) -> u64 {
        self.frames.load(Ordering::Acquire)
    }

    /// The mean time between two presents, in microseconds; 0 with no frame.
    pub fn mean_interval_us(&self) -> u64 {
        self.interval_us.load(Ordering::Acquire).checked_div(self.frames()).unwrap_or(0)
    }

    /// The mean time of the game's `Present` call, in microseconds; 0 with no frame.
    pub fn mean_present_us(&self) -> u64 {
        self.present_us.load(Ordering::Acquire).checked_div(self.frames()).unwrap_or(0)
    }

    pub fn worst_us(&self) -> u64 {
        self.worst_us.load(Ordering::Acquire)
    }

    pub fn reset(&self) {
        self.frames.store(0, Ordering::Release);
        self.interval_us.store(0, Ordering::Release);
        self.present_us.store(0, Ordering::Release);
        self.worst_us.store(0, Ordering::Release);
    }
}
