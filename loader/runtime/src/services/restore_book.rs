use std::sync::{Mutex, MutexGuard};

use crate::structs::RestoreFlag;

/// Remembers the player's own value of each flag that mods set, to write it back when the last mod
/// that set it is released.
///
/// The value before the first set of any mod is kept; later sets only add their mod. The game's
/// value while mods hold the flag is whatever the last of them wrote.
pub struct RestoreBook {
    // Per flag: the value before the first set, and the mods that hold it.
    entries: Mutex<Vec<(RestoreFlag, bool, Vec<usize>)>>,
}

impl Default for RestoreBook {
    fn default() -> Self {
        Self::new()
    }
}

impl RestoreBook {
    pub fn new() -> Self {
        Self { entries: Mutex::new(Vec::new()) }
    }

    fn lock(&self) -> MutexGuard<'_, Vec<(RestoreFlag, bool, Vec<usize>)>> {
        self.entries.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Records that the mod at `owner` set `flag`; `before` is the game's value before this set,
    /// kept only when no mod holds the flag yet.
    pub fn set(&self, owner: usize, flag: RestoreFlag, before: bool) {
        let mut entries = self.lock();
        match entries.iter_mut().find(|(held, _, _)| *held == flag) {
            Some((_, _, owners)) => {
                if !owners.contains(&owner) {
                    owners.push(owner);
                }
            }
            None => entries.push((flag, before, vec![owner])),
        }
    }

    /// Forgets the flags of the mod at `owner`. Gives the values to write back: one per flag that no
    /// other mod holds any more, with the value from before the first set.
    pub fn release_owner(&self, owner: usize) -> Vec<(RestoreFlag, bool)> {
        let mut entries = self.lock();
        let mut restores = Vec::new();
        entries.retain_mut(|(flag, before, owners)| {
            if !owners.contains(&owner) {
                return true;
            }
            owners.retain(|held| *held != owner);
            if owners.is_empty() {
                restores.push((*flag, *before));
                return false;
            }
            true
        });
        restores
    }
}
