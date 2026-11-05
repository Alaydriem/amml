use super::refresh_state::RefreshState;
use crate::RegistryView;
use std::sync::{Condvar, Mutex, PoisonError};

// Ends the refresh in flight when it is dropped, also when the run panics, so no request waits for ever.
pub(crate) struct RefreshEnd<'a> {
    state: &'a Mutex<RefreshState>,
    changed: &'a Condvar,
    id: u64,
    views: Option<Vec<RegistryView>>,
}

impl<'a> RefreshEnd<'a> {
    pub(crate) fn new(state: &'a Mutex<RefreshState>, changed: &'a Condvar, id: u64) -> RefreshEnd<'a> {
        RefreshEnd { state, changed, id, views: None }
    }

    // the views the run gave; without them the refresh ends as one that gave nothing
    pub(crate) fn finish(&mut self, views: Vec<RegistryView>) {
        self.views = Some(views);
    }
}

impl Drop for RefreshEnd<'_> {
    fn drop(&mut self) {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        state.running = None;
        state.done = state.done.max(self.id);
        if let Some(views) = self.views.take() {
            state.views = Some((self.id, views));
        }
        self.changed.notify_all();
    }
}
