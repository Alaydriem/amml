use std::sync::atomic::{AtomicU8, Ordering};

/// Remembers whether the player was in a world at the last look, so a change can be told from a repeat.
pub struct WorldLatch {
    state: AtomicU8,
}

impl WorldLatch {
    const UNKNOWN: u8 = 0;
    const OUT: u8 = 1;
    const IN: u8 = 2;

    /// A latch that has seen nothing yet; the first look counts as a change.
    pub const fn new() -> Self {
        Self { state: AtomicU8::new(Self::UNKNOWN) }
    }

    /// Records `in_world`. True when it differs from the last look.
    pub fn set(&self, in_world: bool) -> bool {
        let next = if in_world { Self::IN } else { Self::OUT };
        self.state.swap(next, Ordering::Relaxed) != next
    }
}

impl Default for WorldLatch {
    fn default() -> Self {
        Self::new()
    }
}
