use std::sync::Once;

use crate::Boundary;

/// Runs the standalone start-up once per process, however many threads reach the exe entry point.
///
/// A caller that arrives while start-up runs waits for it to finish, so no thread continues into
/// the game before the mods are loaded. A panic in start-up is stopped here and counts as done:
/// later callers do not run it again and the game still starts.
pub struct StartupGate {
    once: Once,
}

impl StartupGate {
    pub const fn new() -> Self {
        Self { once: Once::new() }
    }

    /// Runs `start` on the first call. Every call returns only after that run finished.
    pub fn pass(&self, start: impl FnOnce()) {
        self.once.call_once(|| Boundary::run((), start));
    }

    /// Whether start-up already ran.
    pub fn is_done(&self) -> bool {
        self.once.is_completed()
    }
}

impl Default for StartupGate {
    fn default() -> Self {
        Self::new()
    }
}
