use std::sync::{Mutex, MutexGuard};

use crate::CapturedHandler;
use crate::structs::handler_capture_state::HandlerCaptureState;

/// The client network handler of the joined world, captured from the calls of the hooked handler slots.
///
/// Each call records the handler, its `NetworkIdentifier` and the thread of the call. A handler that differs from the
/// one held (or the first after a world leave or a join) is a new capture with a new generation; the generation only
/// grows, so a mod that compares generations sees every new capture. A world leave forgets the handler and refuses
/// captures until the next join starts, so a late call of the left world's handler is not kept for the next world; a
/// join start forgets it too, so a missed world leave does not keep it either. The capture
/// makes no game call and never reads the addresses.
pub struct HandlerCapture {
    state: Mutex<HandlerCaptureState>,
}

impl Default for HandlerCapture {
    fn default() -> Self {
        Self::new()
    }
}

impl HandlerCapture {
    pub fn new() -> Self {
        Self { state: Mutex::new(HandlerCaptureState::default()) }
    }

    // The state holds plain data, so a lock poisoned by a panic is recovered.
    fn lock(&self) -> MutexGuard<'_, HandlerCaptureState> {
        self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// A handler slot was called on `handler` with the identifier at `id`, on the current thread. Gives true when this
    /// is a new capture (a new generation). A null handler, and any handler between a world leave and the next join,
    /// is not captured.
    pub fn capture(&self, handler: usize, id: usize) -> bool {
        if handler == 0 {
            return false;
        }
        let thread = std::thread::current().id();
        let mut state = self.lock();
        if state.closed {
            return false;
        }
        if let Some(held) = state.held.as_mut().filter(|held| held.handler == handler) {
            held.id = id;
            held.thread = thread;
            return false;
        }
        state.generation = state.generation.wrapping_add(1).max(1);
        state.held = Some(CapturedHandler { handler, id, thread, generation: state.generation });
        true
    }

    /// The handler held now; `None` before the first capture of the world.
    pub fn captured(&self) -> Option<CapturedHandler> {
        self.lock().held
    }

    /// Before `WorldLeft`: the handler of the left world is forgotten, and captures are refused until `join_started`.
    pub fn world_left(&self) {
        let mut state = self.lock();
        state.held = None;
        state.closed = true;
    }

    /// A join starts: the handler of an earlier world is forgotten (also when no world leave came before), and the
    /// handler slots capture again.
    pub fn join_started(&self) {
        let mut state = self.lock();
        state.held = None;
        state.closed = false;
    }
}
