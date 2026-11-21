use std::sync::{Mutex, MutexGuard};

use crate::structs::FocusMessage;

#[derive(Default)]
struct HoldState {
    held: Vec<FocusMessage>,
    marks: Vec<FocusMessage>,
}

/// Holds focus-loss messages a listener cancelled, so the game keeps its input state, and replays them on request.
///
/// Held messages are kept oldest first, one per message kind: a second loss of a kind replaces the first in place.
/// A focus gain forgets the held messages and the replay marks, because the game never saw the loss. A gain that
/// comes after `take_replay` and before the posted replay arrives clears its mark too: that replay is then treated
/// as a new loss (`is_replay` false) and a listener may hold it again.
pub struct FocusHold {
    state: Mutex<HoldState>,
}

impl Default for FocusHold {
    fn default() -> Self {
        Self::new()
    }
}

impl FocusHold {
    pub fn new() -> Self {
        Self { state: Mutex::new(HoldState::default()) }
    }

    fn lock(&self) -> MutexGuard<'_, HoldState> {
        self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Records one focus loss. True when the game gets it; false when `cancelled` held it back.
    pub fn loss(&self, message: FocusMessage, cancelled: bool) -> bool {
        if !cancelled {
            return true;
        }
        let mut state = self.lock();
        match state.held.iter_mut().find(|held| held.msg == message.msg) {
            Some(held) => *held = message,
            None => state.held.push(message),
        }
        false
    }

    /// Records a focus gain: the held losses and the replay marks are forgotten.
    pub fn gain(&self) {
        let mut state = self.lock();
        state.held.clear();
        state.marks.clear();
    }

    /// Takes the held messages, oldest first, and marks each for one pass through `is_replay`.
    pub fn take_replay(&self) -> Vec<FocusMessage> {
        let mut state = self.lock();
        let taken = std::mem::take(&mut state.held);
        state.marks.extend(taken.iter().copied());
        taken
    }

    /// True once per mark `take_replay` set for `message`: the replay passes to the game.
    pub fn is_replay(&self, message: &FocusMessage) -> bool {
        let mut state = self.lock();
        match state.marks.iter().position(|mark| mark == message) {
            Some(at) => {
                state.marks.remove(at);
                true
            }
            None => false,
        }
    }
}
