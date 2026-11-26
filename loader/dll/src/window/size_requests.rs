use std::sync::{Mutex, MutexGuard};

#[derive(Default)]
struct RequestState {
    // The mod whose size request is in force; its disable gives the window its own size back.
    requester: Option<usize>,
    // The `WM_SIZE` `lParam` of each size message the loader posted and the window has not run yet, with the mod
    // it was posted for.
    pending: Vec<(isize, usize)>,
}

/// The size requests of mods: who holds the render size, and the size messages the loader posted.
pub struct SizeRequests {
    state: Mutex<RequestState>,
}

impl Default for SizeRequests {
    fn default() -> Self {
        Self::new()
    }
}

impl SizeRequests {
    pub fn new() -> Self {
        Self { state: Mutex::new(RequestState::default()) }
    }

    fn lock(&self) -> MutexGuard<'_, RequestState> {
        self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Records a size message about to be posted for `owner`. `holds` true makes `owner` the requester; false
    /// (the window's own size given back) leaves no requester.
    pub fn posting(&self, owner: usize, lparam: isize, holds: bool) {
        let mut state = self.lock();
        state.requester = holds.then_some(owner);
        state.pending.push((lparam, owner));
    }

    /// Forgets a size message whose post failed.
    pub fn post_failed(&self, owner: usize, lparam: isize) {
        let mut state = self.lock();
        if let Some(at) = state.pending.iter().position(|entry| *entry == (lparam, owner)) {
            state.pending.remove(at);
        }
    }

    /// The mod a loader-posted size message with `lparam` was for, once; `None` for a message the loader did not
    /// post.
    pub fn take_posted(&self, lparam: isize) -> Option<usize> {
        let mut state = self.lock();
        let at = state.pending.iter().position(|(posted, _)| *posted == lparam)?;
        Some(state.pending.remove(at).1)
    }

    /// Forgets the size messages posted to a window that is gone; the requester stays.
    pub fn forget_pending(&self) {
        self.lock().pending.clear();
    }

    /// True when `owner` held the size; from then on it does not.
    pub fn end_owner(&self, owner: usize) -> bool {
        let mut state = self.lock();
        if state.requester == Some(owner) {
            state.requester = None;
            return true;
        }
        false
    }
}
