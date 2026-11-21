use std::sync::{Mutex, MutexGuard};

use amml_abi::AmmlMouseButton;

use crate::structs::ResourceId;

const BUTTONS: usize = 5;

/// Tracks which mouse buttons the real feed holds and which leases hold each one through
/// injection, under the same rules as the keys of `KeyTable`.
///
/// An injected down reaches the game only from the first holder while the real button is up, an
/// injected up only from the last holder while the real button is up. A real up reaches the game
/// and ends every injected hold of that button, so an up from a lease that held the button before
/// the real up is dropped.
pub struct ButtonTable {
    // Per button: the real button is down, and the leases that hold it through injection.
    state: Mutex<Vec<(bool, Vec<ResourceId>)>>,
}

impl Default for ButtonTable {
    fn default() -> Self {
        Self::new()
    }
}

impl ButtonTable {
    pub fn new() -> Self {
        Self { state: Mutex::new(vec![(false, Vec::new()); BUTTONS]) }
    }

    fn lock(&self) -> MutexGuard<'_, Vec<(bool, Vec<ResourceId>)>> {
        self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Records one real button feed and gives the leases whose injected hold a real up ended, in the
    /// order they pressed it. An unknown button changes nothing.
    pub fn real(&self, button: AmmlMouseButton, down: bool) -> Vec<ResourceId> {
        let mut state = self.lock();
        let Some(slot) = state.get_mut(usize::from(button.0)) else {
            return Vec::new();
        };
        slot.0 = down;
        if down { Vec::new() } else { std::mem::take(&mut slot.1) }
    }

    /// Records one injected button from `lease`. True when the game must get it. An unknown button,
    /// a down from a lease that already holds the button and an up from a lease that does not hold it
    /// are dropped.
    pub fn injected(&self, lease: ResourceId, button: AmmlMouseButton, down: bool) -> bool {
        let mut state = self.lock();
        let Some((real, holders)) = state.get_mut(usize::from(button.0)) else {
            return false;
        };
        if down {
            if holders.contains(&lease) {
                return false;
            }
            holders.push(lease);
            holders.len() == 1 && !*real
        } else {
            let Some(position) = holders.iter().position(|holder| *holder == lease) else {
                return false;
            };
            holders.swap_remove(position);
            holders.is_empty() && !*real
        }
    }

    /// Whether `lease` is recorded as an injected holder of `button`.
    pub fn holds(&self, lease: ResourceId, button: AmmlMouseButton) -> bool {
        self.lock().get(usize::from(button.0)).is_some_and(|(_, holders)| holders.contains(&lease))
    }

    /// Whether the real button is down; `None` for an unknown button.
    pub fn is_down(&self, button: AmmlMouseButton) -> Option<bool> {
        self.lock().get(usize::from(button.0)).map(|(real, _)| *real)
    }
}
