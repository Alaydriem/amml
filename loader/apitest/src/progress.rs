use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};

use crate::Step;

/// What the API test mod has seen so far. Listeners may run on different game threads, so every
/// field is atomic.
pub struct Progress {
    steps: AtomicU64,
    ticks: AtomicU32,
    passed: AtomicBool,
}

impl Default for Progress {
    fn default() -> Self {
        Self::new()
    }
}

impl Progress {
    /// The client tick from which the mod checks the other steps.
    pub const TICKS: u32 = 100;

    pub const fn new() -> Self {
        Self { steps: AtomicU64::new(0), ticks: AtomicU32::new(0), passed: AtomicBool::new(false) }
    }

    /// Forgets every step and tick.
    pub fn reset(&self) {
        self.steps.store(0, Ordering::Release);
        self.ticks.store(0, Ordering::Release);
        self.passed.store(false, Ordering::Release);
    }

    /// Marks `step` seen; true the first time only.
    pub fn set(&self, step: Step) -> bool {
        self.steps.fetch_or(step.bit(), Ordering::AcqRel) & step.bit() == 0
    }

    pub fn seen(&self, step: Step) -> bool {
        self.steps.load(Ordering::Acquire) & step.bit() != 0
    }

    /// Counts one client tick and gives the new count.
    pub fn tick(&self) -> u32 {
        self.ticks.fetch_add(1, Ordering::AcqRel).saturating_add(1)
    }

    /// The steps not seen yet, by name, in run order; empty when every step was seen.
    pub fn missing(&self) -> Vec<&'static str> {
        Step::ALL.into_iter().filter(|step| !self.seen(*step)).map(Step::name).collect()
    }

    /// True once, the first time it is called with every step seen and at least `TICKS` ticks.
    pub fn try_pass(&self) -> bool {
        if self.ticks.load(Ordering::Acquire) < Self::TICKS || !self.missing().is_empty() {
            return false;
        }
        !self.passed.swap(true, Ordering::AcqRel)
    }
}
