use std::sync::{Mutex, MutexGuard};

/// Which mods hold an app focus loss that one of their listeners cancelled, so that the game's handler did not run.
///
/// A mod holds a loss from the cancel until an app focus gain (the game then never needs the loss) or until the mod
/// replays it. The book keeps one mark per mod: a second cancel before the replay is the same held loss.
pub struct FocusReplayBook {
    held: Mutex<Vec<usize>>,
}

impl Default for FocusReplayBook {
    fn default() -> Self {
        Self::new()
    }
}

impl FocusReplayBook {
    pub fn new() -> Self {
        Self { held: Mutex::new(Vec::new()) }
    }

    fn lock(&self) -> MutexGuard<'_, Vec<usize>> {
        self.held.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Records that a listener of `owner` cancelled a loss.
    pub fn cancelled(&self, owner: usize) {
        let mut held = self.lock();
        if !held.contains(&owner) {
            held.push(owner);
        }
    }

    /// True while `owner` holds a loss.
    pub fn holds(&self, owner: usize) -> bool {
        self.lock().contains(&owner)
    }

    /// True while any mod holds a loss.
    pub fn any(&self) -> bool {
        !self.lock().is_empty()
    }

    /// A focus gain: every held loss is forgotten. True when one was held.
    pub fn gained(&self) -> bool {
        let mut held = self.lock();
        let had = !held.is_empty();
        held.clear();
        had
    }

    /// Takes the loss of `owner` for a replay: true, and the mark is cleared, when `owner` held one.
    pub fn replay(&self, owner: usize) -> bool {
        let mut held = self.lock();
        match held.iter().position(|mark| *mark == owner) {
            Some(at) => {
                held.remove(at);
                true
            }
            None => false,
        }
    }

    /// Forgets the loss of `owner`, for a mod that is disabled.
    pub fn forget_owner(&self, owner: usize) {
        self.lock().retain(|mark| *mark != owner);
    }
}
