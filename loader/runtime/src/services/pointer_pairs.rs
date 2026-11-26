use std::sync::{Mutex, MutexGuard};

use amml_abi::AmmlMouseButton;

const BUTTONS: usize = 5;

/// Keeps a button's down and up together when a listener cancels the window down.
///
/// A cancelled window down never reaches the game, so its up must not either, and the game's mouse feed copy of
/// that down and up must not be raised. The pairing of one press ends at the next window down of the button.
/// An up whose down reached the game (or was never seen) reaches it too, whatever the listeners say, so the game
/// never keeps a button down.
pub struct PointerPairs {
    // Per button: the window down was cancelled (read by the window up), and the same for the feed copies.
    state: Mutex<Vec<(bool, bool)>>,
}

impl Default for PointerPairs {
    fn default() -> Self {
        Self::new()
    }
}

impl PointerPairs {
    pub fn new() -> Self {
        Self { state: Mutex::new(vec![(false, false); BUTTONS]) }
    }

    fn lock(&self) -> MutexGuard<'_, Vec<(bool, bool)>> {
        self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Records one window button message. True when the game gets it. A down follows `cancelled`; an up follows
    /// its down and ignores `cancelled`. An unknown button follows `cancelled` alone.
    pub fn window_button(&self, button: AmmlMouseButton, down: bool, cancelled: bool) -> bool {
        let mut state = self.lock();
        let Some(slot) = state.get_mut(usize::from(button.0)) else {
            return !cancelled;
        };
        if down {
            *slot = (cancelled, cancelled);
            !cancelled
        } else {
            let paired = slot.0;
            slot.0 = false;
            !paired
        }
    }

    /// Forgets every pairing, for a new window: no button counts as cancelled.
    pub fn reset(&self) {
        self.lock().iter_mut().for_each(|slot| *slot = (false, false));
    }

    /// Records one copy of a button in the game's mouse feed. True when the feed goes on: false for the down and
    /// the up of a button whose window down was cancelled.
    pub fn feed_button(&self, button: AmmlMouseButton, down: bool) -> bool {
        let mut state = self.lock();
        let Some(slot) = state.get_mut(usize::from(button.0)) else {
            return true;
        };
        let pass = !slot.1;
        if !down {
            slot.1 = false;
        }
        pass
    }
}
