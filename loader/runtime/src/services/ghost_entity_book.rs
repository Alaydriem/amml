use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard};

use crate::structs::ghost_entity_book_state::GhostEntityBookState;
use crate::structs::ghost_entity_entry::GhostEntityEntry;
use crate::{GhostEntity, GhostEntityKey, GhostEntityRefusal};

/// The block entity ghosts of the loader: an id with a generation and an owner mod for each, and the game actors the
/// render thread must release.
///
/// A ghost is a game block actor the loader owns, bound to one mod and one world. The book makes no game call. A call
/// that ends the life of a ghost (`free`, `disable_owner`, `world_left`, `device_reset`) removes it from the book and
/// queues its actor: from then on every use of the id gives `Stale`, and `take_due` gives the queue to the render
/// thread, which releases each actor once at its next `renderLevel` call.
///
/// Ids come from one counter that never goes back, so an id is never given twice; a generation changes at each world
/// leave and device reset.
pub struct GhostEntityBook {
    state: Mutex<GhostEntityBookState>,
    // True while `due` holds an actor; the render detour reads it without the lock.
    pending: AtomicBool,
}

impl Default for GhostEntityBook {
    fn default() -> Self {
        Self::new()
    }
}

impl GhostEntityBook {
    /// The most live ghosts one mod holds.
    pub const MAX_PER_MOD: usize = 256;

    pub fn new() -> Self {
        Self {
            state: Mutex::new(GhostEntityBookState {
                next_id: 1,
                generation: 0,
                entries: std::collections::BTreeMap::new(),
                due: Vec::new(),
            }),
            pending: AtomicBool::new(false),
        }
    }

    // The state holds plain data, so a lock poisoned by a panic is recovered.
    fn lock(&self) -> MutexGuard<'_, GhostEntityBookState> {
        self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Whether `owner` holds fewer than `MAX_PER_MOD` ghosts, so a `create` now is not refused for the cap.
    pub fn has_room(&self, owner: usize) -> bool {
        Self::count_of(&self.lock(), owner) < Self::MAX_PER_MOD
    }

    /// Adds `ghost` as a ghost of `owner`. `TooMany` when `owner` holds `MAX_PER_MOD` ghosts; the book then does not
    /// keep the actor, and the caller still owns it.
    pub fn create(&self, owner: usize, ghost: GhostEntity) -> Result<GhostEntityKey, GhostEntityRefusal> {
        let mut state = self.lock();
        if Self::count_of(&state, owner) >= Self::MAX_PER_MOD {
            return Err(GhostEntityRefusal::TooMany);
        }
        let id = state.next_id;
        state.next_id = id.checked_add(1).ok_or(GhostEntityRefusal::TooMany)?;
        state.entries.insert(id, GhostEntityEntry { owner, ghost });
        Ok(GhostEntityKey { id, generation: state.generation })
    }

    /// The game side of the ghost `key` of `owner`.
    pub fn entry(&self, owner: usize, key: GhostEntityKey) -> Result<GhostEntity, GhostEntityRefusal> {
        let state = self.lock();
        let entry = Self::entry_of(&state, key)?;
        if entry.owner == owner { Ok(entry.ghost) } else { Err(GhostEntityRefusal::NotOwner) }
    }

    /// Ends the ghost `key` of `owner` and queues its actor.
    pub fn free(&self, owner: usize, key: GhostEntityKey) -> Result<(), GhostEntityRefusal> {
        let mut state = self.lock();
        let entry = Self::entry_of(&state, key)?;
        if entry.owner != owner {
            return Err(GhostEntityRefusal::NotOwner);
        }
        if let Some(entry) = state.entries.remove(&key.id) {
            state.due.push(entry.ghost);
        }
        self.mark(&state);
        Ok(())
    }

    /// Drops every ghost of `owner` and queues their actors. Gives the number dropped.
    pub fn disable_owner(&self, owner: usize) -> usize {
        let mut state = self.lock();
        let ids: Vec<u32> = state.entries.iter().filter(|(_, entry)| entry.owner == owner).map(|(id, _)| *id).collect();
        for id in &ids {
            if let Some(entry) = state.entries.remove(id) {
                state.due.push(entry.ghost);
            }
        }
        self.mark(&state);
        ids.len()
    }

    /// The world is left: every ghost is dropped, its actor queued, and the generation changes. Gives the number dropped.
    pub fn world_left(&self) -> usize {
        self.invalidate()
    }

    /// The render device was reset or the surface lost: as `world_left`.
    pub fn device_reset(&self) -> usize {
        self.invalidate()
    }

    /// The actors to release now, each once; the queue is empty after it. Called on the render thread.
    pub fn take_due(&self) -> Vec<GhostEntity> {
        let mut state = self.lock();
        self.pending.store(false, Ordering::Release);
        std::mem::take(&mut state.due)
    }

    /// Whether an actor waits to be released. One atomic load, no lock.
    pub fn has_due(&self) -> bool {
        self.pending.load(Ordering::Acquire)
    }

    /// The number of live ghosts.
    pub fn len(&self) -> usize {
        self.lock().entries.len()
    }

    /// Whether the book holds no live ghost.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn count_of(state: &GhostEntityBookState, owner: usize) -> usize {
        state.entries.values().filter(|entry| entry.owner == owner).count()
    }

    fn invalidate(&self) -> usize {
        let mut state = self.lock();
        let entries = std::mem::take(&mut state.entries);
        let count = entries.len();
        state.due.extend(entries.into_values().map(|entry| entry.ghost));
        state.generation = state.generation.wrapping_add(1);
        self.mark(&state);
        count
    }

    fn mark(&self, state: &GhostEntityBookState) {
        if !state.due.is_empty() {
            self.pending.store(true, Ordering::Release);
        }
    }

    fn entry_of(state: &GhostEntityBookState, key: GhostEntityKey) -> Result<&GhostEntityEntry, GhostEntityRefusal> {
        if key.generation != state.generation {
            return Err(GhostEntityRefusal::Stale);
        }
        state.entries.get(&key.id).ok_or(GhostEntityRefusal::Stale)
    }
}
