use super::Locked;
use crate::JobKind;
use std::cell::Cell;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

// Sets the busy flag back to false and forgets the running job, once: by `release`, or when it drops if `release` did
// not run. After `release` a drop does nothing, so it cannot free the next job.
pub(crate) struct BusyReset<'a> {
    busy: &'a AtomicBool,
    current: &'a Mutex<Option<JobKind>>,
    armed: Cell<bool>,
}

impl<'a> BusyReset<'a> {
    pub(crate) fn new(busy: &'a AtomicBool, current: &'a Mutex<Option<JobKind>>) -> BusyReset<'a> {
        BusyReset { busy, current, armed: Cell::new(true) }
    }

    pub(crate) fn release(&self) {
        if self.armed.replace(false) {
            *Locked::of(self.current) = None;
            self.busy.store(false, Ordering::SeqCst);
        }
    }
}

impl Drop for BusyReset<'_> {
    fn drop(&mut self) {
        self.release();
    }
}
