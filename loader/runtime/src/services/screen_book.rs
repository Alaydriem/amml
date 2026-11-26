use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};

use crate::structs::{ScreenBookState, ViewEntry};

/// Gives ids to screen controllers and views, and retires them.
///
/// A controller address gets a screen id at the first event that names it, and a view address a view id the same
/// way. Ids come from one counter that only goes up, so an id is never given twice: a controller or view address
/// that the game reuses after the controller was freed gets a new id. `retire` ends a controller and its views.
/// An id is never zero.
pub struct ScreenBook {
    state: Mutex<ScreenBookState>,
}

impl Default for ScreenBook {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenBook {
    pub fn new() -> Self {
        Self { state: Mutex::new(ScreenBookState { next: 1, screens: HashMap::new(), views: HashMap::new() }) }
    }

    // The state holds plain data, so a lock poisoned by a panic is recovered.
    fn lock(&self) -> MutexGuard<'_, ScreenBookState> {
        self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// The screen id of `controller` and the view id of `view`, given at the first call. A view that now
    /// belongs to another controller than before gets a new view id.
    pub fn note(&self, controller: usize, view: usize) -> (u64, u64) {
        let mut state = self.lock();
        let screen = Self::screen_in(&mut state, controller);
        let view_id = match state.views.get(&view) {
            Some(entry) if entry.controller == controller => entry.id,
            _ => {
                let id = Self::take(&mut state);
                state.views.insert(view, ViewEntry { id, controller });
                id
            }
        };
        (screen, view_id)
    }

    /// The screen id of `controller`, given now when it has none: an event that names a controller before its first
    /// draw (a hover in the first frame of a screen) gives the id, and the draw keeps it.
    pub fn note_controller(&self, controller: usize) -> u64 {
        Self::screen_in(&mut self.lock(), controller)
    }

    fn screen_in(state: &mut ScreenBookState, controller: usize) -> u64 {
        if let Some(id) = state.screens.get(&controller) {
            return *id;
        }
        let id = Self::take(state);
        state.screens.insert(controller, id);
        id
    }

    /// Ends `controller` and every view of it. Gives the screen id it had, or `None` for a controller the book
    /// never gave an id: no closing is raised for those.
    pub fn retire(&self, controller: usize) -> Option<u64> {
        let mut state = self.lock();
        let screen = state.screens.remove(&controller)?;
        state.views.retain(|_, entry| entry.controller != controller);
        Some(screen)
    }

    /// The address of the live view with id `view`; `None` for a retired or unknown id.
    pub fn view_address(&self, view: u64) -> Option<usize> {
        self.lock().views.iter().find(|(_, entry)| entry.id == view).map(|(address, _)| *address)
    }

    /// The screen id of the live controller of the view with id `view`.
    pub fn screen_of_view(&self, view: u64) -> Option<u64> {
        let state = self.lock();
        let entry = state.views.values().find(|entry| entry.id == view)?;
        state.screens.get(&entry.controller).copied()
    }

    /// The screen id of the live `controller`; `None` for a controller the book never gave an id.
    pub fn screen_of_controller(&self, controller: usize) -> Option<u64> {
        self.lock().screens.get(&controller).copied()
    }

    /// The address of the live controller of the view with id `view`.
    pub fn controller_of_view(&self, view: u64) -> Option<usize> {
        self.lock().views.values().find(|entry| entry.id == view).map(|entry| entry.controller)
    }

    fn take(state: &mut ScreenBookState) -> u64 {
        let id = state.next;
        state.next += 1;
        id
    }
}
