use std::sync::atomic::{AtomicBool, Ordering};

/// Marks the time `FramePresent` listeners run with a surface lent to them: the flag is set while the value lives
/// and cleared when it drops, also when a panic unwinds through the listeners.
pub struct DrawWindow<'a> {
    flag: &'a AtomicBool,
}

impl<'a> DrawWindow<'a> {
    pub fn open(flag: &'a AtomicBool) -> Self {
        flag.store(true, Ordering::Release);
        Self { flag }
    }
}

impl Drop for DrawWindow<'_> {
    fn drop(&mut self) {
        self.flag.store(false, Ordering::Release);
    }
}
