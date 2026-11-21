use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard};

/// The one server join that waits for the game's answer: its owner (the mod index) and its id.
///
/// The game's result callable is static and has no state, so one join waits at a time. `begin` refuses a second
/// join while one waits; `answer` takes the waiting join, once, from any thread.
#[derive(Debug)]
pub struct JoinBook {
    waiting: Mutex<Option<(usize, u64)>>,
    next: AtomicU64,
}

impl JoinBook {
    pub fn new() -> Self {
        Self { waiting: Mutex::new(None), next: AtomicU64::new(1) }
    }

    /// A new join id for `owner`, now waiting; `None` while another join waits.
    pub fn begin(&self, owner: usize) -> Option<u64> {
        let mut waiting = self.lock();
        if waiting.is_some() {
            return None;
        }
        let id = self.next.fetch_add(1, Ordering::Relaxed);
        *waiting = Some((owner, id));
        Some(id)
    }

    /// Takes the waiting join back without an answer (the game was not called).
    pub fn cancel(&self, id: u64) {
        let mut waiting = self.lock();
        if waiting.is_some_and(|(_, waiting)| waiting == id) {
            *waiting = None;
        }
    }

    /// The waiting join (owner, id), taken; `None` when no join waits.
    pub fn answer(&self) -> Option<(usize, u64)> {
        self.lock().take()
    }

    /// Whether a join waits.
    pub fn is_waiting(&self) -> bool {
        self.lock().is_some()
    }

    // The slot holds plain data, so a lock poisoned by a panic is recovered.
    fn lock(&self) -> MutexGuard<'_, Option<(usize, u64)>> {
        self.waiting.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl Default for JoinBook {
    fn default() -> Self {
        Self::new()
    }
}
