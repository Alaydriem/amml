use std::collections::BTreeSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard};

/// The wishes of the mods to hide the local player's body: each mod has one wish, and any wish hides the body.
///
/// A wish ends by `set(owner, false)`, the mod's disable and the world leave. The book makes no game call and any
/// thread may use it. `hidden` reads one atomic flag without the lock, for the render detours.
#[derive(Debug, Default)]
pub struct HideBook {
    wishes: Mutex<BTreeSet<usize>>,
    hidden: AtomicBool,
}

impl HideBook {
    pub fn new() -> Self {
        Self { wishes: Mutex::new(BTreeSet::new()), hidden: AtomicBool::new(false) }
    }

    // The set holds plain data, so a lock poisoned by a panic is recovered.
    fn lock(&self) -> MutexGuard<'_, BTreeSet<usize>> {
        self.wishes.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// The mod `owner` wishes the body hidden (`on`) or shown. Gives true when any wish exists after the call.
    pub fn set(&self, owner: usize, on: bool) -> bool {
        let mut wishes = self.lock();
        if on {
            wishes.insert(owner);
        } else {
            wishes.remove(&owner);
        }
        let any = !wishes.is_empty();
        self.hidden.store(any, Ordering::Release);
        any
    }

    /// Whether any mod wishes the body hidden: one relaxed load.
    pub fn hidden(&self) -> bool {
        self.hidden.load(Ordering::Relaxed)
    }

    /// The mod `owner` is disabled: true when it had a wish.
    pub fn disable(&self, owner: usize) -> bool {
        let mut wishes = self.lock();
        let had = wishes.remove(&owner);
        self.hidden.store(!wishes.is_empty(), Ordering::Release);
        had
    }

    /// Before `WorldLeft`: every wish ends. True when any existed.
    pub fn world_left(&self) -> bool {
        let mut wishes = self.lock();
        let had = !wishes.is_empty();
        wishes.clear();
        self.hidden.store(false, Ordering::Release);
        had
    }
}
