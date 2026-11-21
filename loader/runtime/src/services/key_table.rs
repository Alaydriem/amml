use std::sync::{Mutex, MutexGuard};

use amml_abi::AmmlModifiers;

use crate::structs::{KeyVerdict, ResourceId};

const KEYS: usize = 0x100;

// Virtual key to modifier bits: the generic keys, then the side keys.
const MODIFIER_BITS: [(usize, AmmlModifiers); 11] = [
    (0x10, AmmlModifiers::SHIFT),
    (0x11, AmmlModifiers::CTRL),
    (0x12, AmmlModifiers::ALT),
    (0xA0, AmmlModifiers::LSHIFT),
    (0xA1, AmmlModifiers::RSHIFT),
    (0xA2, AmmlModifiers::LCTRL),
    (0xA3, AmmlModifiers::RCTRL),
    (0xA4, AmmlModifiers::LALT),
    (0xA5, AmmlModifiers::RALT),
    (0x5B, AmmlModifiers::LWIN),
    (0x5C, AmmlModifiers::RWIN),
];

#[derive(Clone, Default)]
struct KeyState {
    // The user's hands: the real feed only.
    down: bool,
    // The leases that hold the key through injection; the game gets a down from the first and an up
    // from the last, and neither while the real key is down. A real up clears it.
    holders: Vec<ResourceId>,
    // The first down of the current press has been answered by the listeners.
    decided: bool,
    // The first down of this key was cancelled, so the game never saw it.
    down_cancelled: bool,
}

impl KeyState {
    // The real key is down in the game: the user holds it and its first down was not held back.
    fn down_in_game(&self) -> bool {
        self.down && !self.down_cancelled
    }
}

struct TableState {
    keys: Vec<KeyState>,
    seen: bool,
}

/// Tracks which keys are down from what the loader saw on the real feed.
///
/// `is_down` and `modifiers` follow the real feed, not what the game got: a
/// cancelled down still counts as down, and an injected key never shows in
/// them. Each key also records which leases hold it through injection, so an
/// up reaches the game only when no other holder (another lease or the real
/// key) still holds it. A real up reaches the game and ends every injected
/// hold of that key: the game now sees the key up, so an up from a lease that
/// held the key before the real up is dropped, even when another lease has
/// pressed the key since.
pub struct KeyTable {
    state: Mutex<TableState>,
}

impl Default for KeyTable {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyTable {
    pub fn new() -> Self {
        Self { state: Mutex::new(TableState { keys: vec![KeyState::default(); KEYS], seen: false }) }
    }

    fn lock(&self) -> MutexGuard<'_, TableState> {
        self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Records one real key feed. Call it before the listeners. The verdict names the leases whose
    /// injected hold a real up ended, so their owners can be told.
    pub fn real(&self, key: u16, down: bool) -> KeyVerdict {
        let index = usize::from(key);
        if index >= KEYS {
            return KeyVerdict { raise: false, repeat: false, ended: Vec::new() };
        }
        let mut state = self.lock();
        state.seen = true;
        let slot = &mut state.keys[index];
        let was_down = slot.down;
        slot.down = down;
        if down && !was_down {
            slot.decided = false;
        }
        // A real up ends the injected holds only when its press reached the game; the up of a held-back
        // press is held back too, so the game still has the injected key down.
        let ended = if !down && !slot.down_cancelled { std::mem::take(&mut slot.holders) } else { Vec::new() };
        // `repeat` follows the real hands: an injected hold does not make it true.
        if down {
            KeyVerdict { raise: true, repeat: was_down, ended }
        } else {
            KeyVerdict { raise: was_down, repeat: false, ended }
        }
    }

    /// Records the listeners' answer. Gives whether the game gets the key.
    ///
    /// The first down of a press decides what the game sees of the whole hold:
    /// a cancelled first down holds back its repeats and its up. A cancelled
    /// repeat of a passed down drops only that repeat. A cancel of an up whose
    /// down passed is ignored.
    pub fn finish(&self, key: u16, down: bool, cancelled: bool) -> bool {
        let index = usize::from(key);
        if index >= KEYS {
            return true;
        }
        let mut state = self.lock();
        let slot = &mut state.keys[index];
        if down {
            if slot.decided {
                !(cancelled || slot.down_cancelled)
            } else {
                slot.decided = true;
                slot.down_cancelled = cancelled;
                !cancelled
            }
        } else {
            let held_back = slot.down_cancelled;
            slot.down_cancelled = false;
            slot.decided = false;
            !held_back
        }
    }

    /// Records one injected key from `lease` (one down and one up per lease
    /// and key). True when the game must get it: the first injected down while
    /// the real key is not down in the game, or the last injected up while the
    /// real key is not down in the game (a real press that a lease or listener
    /// held back is not down in the game). A
    /// down from a lease that already holds the key, and an up from a lease
    /// that does not hold it (its hold was ended by a real up), are dropped.
    /// Only the injected holders change.
    pub fn injected(&self, lease: ResourceId, key: u16, down: bool) -> bool {
        let index = usize::from(key);
        if index >= KEYS {
            return false;
        }
        let mut state = self.lock();
        state.seen = true;
        let slot = &mut state.keys[index];
        if down {
            if slot.holders.contains(&lease) {
                return false;
            }
            slot.holders.push(lease);
            slot.holders.len() == 1 && !slot.down_in_game()
        } else {
            let Some(position) = slot.holders.iter().position(|holder| *holder == lease) else {
                return false;
            };
            slot.holders.swap_remove(position);
            slot.holders.is_empty() && !slot.down_in_game()
        }
    }

    /// Whether `lease` is recorded as an injected holder of `key`.
    pub fn holds(&self, lease: ResourceId, key: u16) -> bool {
        self.lock().keys.get(usize::from(key)).is_some_and(|slot| slot.holders.contains(&lease))
    }

    /// Whether the key is down. `None` before the first event.
    pub fn is_down(&self, key: u16) -> Option<bool> {
        let state = self.lock();
        if !state.seen {
            return None;
        }
        state.keys.get(usize::from(key)).map(|slot| slot.down)
    }

    /// The keys the real feed holds down, in key order. A cancelled down counts; an injected hold does not.
    pub fn real_down_keys(&self) -> Vec<u16> {
        let state = self.lock();
        (0u16..).zip(&state.keys).filter(|(_, slot)| slot.down).map(|(key, _)| key).collect()
    }

    pub fn modifiers(&self) -> AmmlModifiers {
        let state = self.lock();
        let bits = MODIFIER_BITS
            .iter()
            .filter(|(key, _)| state.keys[*key].down)
            .fold(0u16, |bits, (_, modifier)| bits | modifier.0);
        AmmlModifiers(bits)
    }

    pub fn seen_any(&self) -> bool {
        self.lock().seen
    }
}
