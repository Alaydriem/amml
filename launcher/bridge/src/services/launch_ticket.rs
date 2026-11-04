use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

// A launch in progress. While one lives, no job starts. Dropping it ends the launch.
pub struct LaunchTicket {
    launching: Arc<AtomicU32>,
}

impl LaunchTicket {
    pub(crate) fn new(launching: &Arc<AtomicU32>) -> LaunchTicket {
        launching.fetch_add(1, Ordering::SeqCst);
        LaunchTicket { launching: Arc::clone(launching) }
    }
}

impl Drop for LaunchTicket {
    fn drop(&mut self) {
        self.launching.fetch_sub(1, Ordering::SeqCst);
    }
}
