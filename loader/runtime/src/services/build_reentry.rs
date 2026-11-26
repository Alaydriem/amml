use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};
use std::thread::ThreadId;

use amml_abi::AmmlOrigin;

use crate::structs::ActionScope;

/// The mod actions that run on each thread, innermost last, so an event the game raises inside an action knows the
/// mod that started it.
///
/// A mod action (`build_at`) opens a scope with `enter` before it calls the game and the scope ends when the returned
/// `ActionScope` drops, also when a panic unwinds past it. An event raised on the same thread while a scope is open
/// gets the origin `THIS_MOD` for the listeners of the innermost action's mod and `OTHER_MOD` for the others; an event
/// with no open scope on its thread comes from the game. Scopes of other threads are not seen.
pub struct BuildReentry {
    stacks: Mutex<HashMap<ThreadId, Vec<usize>>>,
}

impl Default for BuildReentry {
    fn default() -> Self {
        Self::new()
    }
}

impl BuildReentry {
    pub fn new() -> Self {
        Self { stacks: Mutex::new(HashMap::new()) }
    }

    /// Opens an action of the mod at `owner` on this thread, until the scope drops.
    pub fn enter(&self, owner: usize) -> ActionScope<'_> {
        self.lock().entry(std::thread::current().id()).or_default().push(owner);
        ActionScope { reentry: self }
    }

    /// The mod of the innermost action open on this thread; `None` when no action is open here.
    pub fn actor(&self) -> Option<usize> {
        self.lock().get(&std::thread::current().id()).and_then(|stack| stack.last().copied())
    }

    /// The origin of an event raised on this thread for a listener of the mod at `listener`.
    pub fn origin_for(&self, listener: usize) -> AmmlOrigin {
        match self.actor() {
            None => AmmlOrigin::GAME,
            Some(owner) if owner == listener => AmmlOrigin::THIS_MOD,
            Some(_) => AmmlOrigin::OTHER_MOD,
        }
    }

    // Ends the innermost action of this thread.
    pub(crate) fn leave(&self) {
        let mut stacks = self.lock();
        let thread = std::thread::current().id();
        if let Some(stack) = stacks.get_mut(&thread) {
            stack.pop();
            if stack.is_empty() {
                stacks.remove(&thread);
            }
        }
    }

    fn lock(&self) -> MutexGuard<'_, HashMap<ThreadId, Vec<usize>>> {
        self.stacks.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
