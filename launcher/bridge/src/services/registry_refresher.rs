use super::refresh_end::RefreshEnd;
use super::refresh_state::RefreshState;
use crate::RegistryView;
use std::sync::{Condvar, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

// One registry refresh at a time. A request joins the refresh in flight when that refresh started after the last
// settings change; otherwise it waits for that refresh to end and runs a new one. The lock is never held while a
// refresh runs.
pub struct RegistryRefresher {
    state: Mutex<RefreshState>,
    changed: Condvar,
}

impl RegistryRefresher {
    // the time between two refreshes in the background
    pub const INTERVAL: Duration = Duration::from_secs(30 * 60);

    pub fn new() -> RegistryRefresher {
        RegistryRefresher { state: Mutex::new(RefreshState::new()), changed: Condvar::new() }
    }

    // Marks the refresh in flight as stale: the next request runs a new one.
    pub fn settings_changed(&self) {
        self.lock().generation += 1;
    }

    // Runs `run`, or gives the views of a refresh in flight that started after the last settings change.
    pub fn refresh(&self, run: impl FnOnce() -> Vec<RegistryView>) -> Vec<RegistryView> {
        let mut state = self.lock();
        loop {
            match state.running {
                Some((id, generation)) if generation == state.generation => {
                    while state.done < id {
                        state = self.changed.wait(state).unwrap_or_else(PoisonError::into_inner);
                    }
                    // a refresh that ended later started later, so its views are as new as the joined one's
                    if let Some((ended, views)) = &state.views
                        && *ended >= id
                    {
                        return views.clone();
                    }
                    // the joined refresh panicked; run again
                }
                Some(_) => {
                    while state.running.is_some_and(|(_, generation)| generation != state.generation) {
                        state = self.changed.wait(state).unwrap_or_else(PoisonError::into_inner);
                    }
                }
                None => {
                    state.last_id += 1;
                    let id = state.last_id;
                    state.running = Some((id, state.generation));
                    drop(state);
                    let mut end = RefreshEnd::new(&self.state, &self.changed, id);
                    let views = run();
                    end.finish(views.clone());
                    return views;
                }
            }
        }
    }

    fn lock(&self) -> MutexGuard<'_, RefreshState> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl Default for RegistryRefresher {
    fn default() -> RegistryRefresher {
        RegistryRefresher::new()
    }
}
