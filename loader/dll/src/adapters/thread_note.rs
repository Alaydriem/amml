use std::sync::atomic::{AtomicU8, Ordering};

use amml_abi::AmmlThread;

/// Says when an adapter writes its "thread" log line again: for the first call, and for each later call on
/// another thread than the call before it, at most `MAX_LINES` times per process.
pub struct ThreadNote {
    // The thread of the last call plus one; 0 before the first call.
    last: AtomicU8,
    lines: AtomicU8,
}

impl ThreadNote {
    pub const MAX_LINES: u8 = 4;

    pub const fn new() -> Self {
        Self { last: AtomicU8::new(0), lines: AtomicU8::new(0) }
    }

    /// True when a call on `thread` is to be logged: the thread differs from the last call's and fewer than
    /// `MAX_LINES` lines were written.
    pub fn changed(&self, thread: AmmlThread) -> bool {
        let code = thread.0.saturating_add(1);
        if self.last.swap(code, Ordering::AcqRel) == code {
            return false;
        }
        self.lines.fetch_update(Ordering::AcqRel, Ordering::Acquire, |n| (n < Self::MAX_LINES).then_some(n + 1)).is_ok()
    }
}

impl Default for ThreadNote {
    fn default() -> Self {
        Self::new()
    }
}
