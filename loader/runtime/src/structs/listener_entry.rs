use std::sync::{Condvar, Mutex, MutexGuard};
use std::thread::{self, ThreadId};

use super::{EntryState, ListenerId};

/// One registered listener and the calls of it in progress.
#[derive(Debug)]
pub(crate) struct ListenerEntry<L> {
    pub(crate) id: ListenerId,
    pub(crate) owner: usize,
    pub(crate) band: u8,
    pub(crate) listener: L,
    state: Mutex<EntryState>,
    idle: Condvar,
}

impl<L> ListenerEntry<L> {
    pub(crate) fn new(id: ListenerId, owner: usize, band: u8, listener: L) -> Self {
        Self { id, owner, band, listener, state: Mutex::new(EntryState::default()), idle: Condvar::new() }
    }

    fn lock(&self) -> MutexGuard<'_, EntryState> {
        self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Registers a call on this thread. Gives false when the listener was removed.
    pub(crate) fn begin_call(&self) -> bool {
        let mut state = self.lock();
        if state.removed {
            return false;
        }
        state.calls.push(thread::current().id());
        true
    }

    /// Ends the call that `begin_call` registered on this thread.
    pub(crate) fn end_call(&self) {
        let me = thread::current().id();
        let mut state = self.lock();
        if let Some(at) = state.calls.iter().position(|id| *id == me) {
            state.calls.remove(at);
        }
        self.idle.notify_all();
    }

    /// True when no call of this listener is in progress.
    pub(crate) fn is_idle(&self) -> bool {
        self.lock().calls.is_empty()
    }

    /// Waits until no call runs on a thread other than the calling one.
    pub(crate) fn wait_idle(&self) {
        let me: ThreadId = thread::current().id();
        let mut state = self.lock();
        while state.calls.iter().any(|id| *id != me) {
            state = self.idle.wait(state).unwrap_or_else(|poisoned| poisoned.into_inner());
        }
    }

    /// Stops new calls. Gives false when the listener was already closed.
    pub(crate) fn close(&self) -> bool {
        let mut state = self.lock();
        let fresh = !state.removed;
        state.removed = true;
        fresh
    }
}
