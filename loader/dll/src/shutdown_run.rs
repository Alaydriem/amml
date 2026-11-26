use crate::ShutdownLatch;

/// The one shutdown run of the process, from `ShutdownLatch::run`. Dropping it finishes the shutdown and wakes the
/// window thread that waits for it, also when the run panicked.
#[derive(Debug)]
pub struct ShutdownRun<'a> {
    latch: &'a ShutdownLatch,
}

impl<'a> ShutdownRun<'a> {
    pub(crate) fn new(latch: &'a ShutdownLatch) -> Self {
        Self { latch }
    }
}

impl Drop for ShutdownRun<'_> {
    fn drop(&mut self) {
        self.latch.finish();
    }
}
