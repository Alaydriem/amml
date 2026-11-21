use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

use amml_abi::AmmlScreen;

/// Holds the title back while the game leaves a world.
///
/// The game's exit-level call comes first in a leave. For some frames after it the menu page under the world is on
/// top, then the game shows its save screen, then its own page after a leave. A menu call made before the save
/// screen is lost when the game moves to that page. So after `begin`, a `START` read is `LOADING` until a `LOADING`
/// screen was read, or until `LIMIT` after the leave when none was read.
#[derive(Debug)]
pub struct LeaveHold {
    // The time of the leave and whether a loading screen was read since; `None` when no leave is held.
    leave: Mutex<Option<(Instant, bool)>>,
}

impl LeaveHold {
    /// How long a leave holds the title when no loading screen is read.
    pub const LIMIT: Duration = Duration::from_secs(3);

    pub fn new() -> Self {
        Self { leave: Mutex::new(None) }
    }

    /// A world leave started at `now`.
    pub fn begin(&self, now: Instant) {
        *self.lock() = Some((now, false));
    }

    /// The screen a reader gets for the mapped `screen` read at `now`. Only `START` changes, to `LOADING`, while
    /// the leave is held.
    pub fn settle(&self, screen: AmmlScreen, now: Instant) -> AmmlScreen {
        let mut leave = self.lock();
        let Some((since, loading_seen)) = *leave else {
            return screen;
        };
        if screen == AmmlScreen::LOADING {
            *leave = Some((since, true));
            return screen;
        }
        if screen != AmmlScreen::START {
            return screen;
        }
        if loading_seen || now.saturating_duration_since(since) >= Self::LIMIT {
            *leave = None;
            return screen;
        }
        AmmlScreen::LOADING
    }

    // The slot holds plain data, so a lock poisoned by a panic is recovered.
    fn lock(&self) -> MutexGuard<'_, Option<(Instant, bool)>> {
        self.leave.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl Default for LeaveHold {
    fn default() -> Self {
        Self::new()
    }
}
