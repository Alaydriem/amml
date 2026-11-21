use std::collections::HashSet;
use std::sync::{Mutex, MutexGuard};

/// Keeps the up of a UI pointer button together with its down when a listener cancels the down.
///
/// A cancelled down never reaches the game, so its up must not either. The hold-back of one press is per view and
/// pointer, and ends with the up it covers or at the next down of the same pointer in that view (a release that was
/// lost, for example by a focus change, never leaves a stale hold-back). An up whose down reached the game, or
/// that the gate saw no down for, always reaches the game, so the game never keeps a button down.
pub struct PointerUpGate {
    // The (view, pointer) pairs whose down was cancelled and whose up has not come yet.
    held: Mutex<HashSet<(u64, u8)>>,
}

impl Default for PointerUpGate {
    fn default() -> Self {
        Self::new()
    }
}

impl PointerUpGate {
    /// The most hold-backs kept. A full book is cleared before it takes a new one: an up it forgot reaches the
    /// game, which treats it as the up of a button that is not down.
    pub const MAX_HELD: usize = 64;

    pub fn new() -> Self {
        Self { held: Mutex::new(HashSet::new()) }
    }

    fn lock(&self) -> MutexGuard<'_, HashSet<(u64, u8)>> {
        self.held.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// The pointer index of the game's button id `button` in its pointer button list `ids`; `None` for an id that
    /// is not a pointer button.
    pub fn pointer_of(ids: [u32; 2], button: u32) -> Option<u8> {
        ids.iter().position(|id| *id == button).and_then(|index| u8::try_from(index).ok())
    }

    /// Records one down of `pointer` in `view`. A new down ends any hold-back of the pair. True when the game gets
    /// it (`cancelled` is false).
    pub fn down(&self, view: u64, pointer: u8, cancelled: bool) -> bool {
        let mut held = self.lock();
        held.remove(&(view, pointer));
        if cancelled {
            if held.len() >= Self::MAX_HELD {
                held.clear();
            }
            held.insert((view, pointer));
        }
        !cancelled
    }

    /// Records one up of `pointer` in `view`. True when its down was cancelled: the event is raised cancelled and
    /// the game does not get the up. The hold-back ends here.
    pub fn up(&self, view: u64, pointer: u8) -> bool {
        self.lock().remove(&(view, pointer))
    }

    /// Forgets every hold-back. Called when no listener of the event is left: an up that reaches the game then
    /// never passes the gate, so a hold-back kept past that point would catch a later, unrelated up.
    pub fn clear(&self) {
        self.lock().clear();
    }

    /// The number of hold-backs kept now.
    pub fn held(&self) -> usize {
        self.lock().len()
    }
}
