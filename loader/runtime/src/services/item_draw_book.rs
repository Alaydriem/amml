use std::collections::{BTreeMap, HashMap};
use std::sync::{Mutex, MutexGuard};

use crate::structs::item_draw_book_state::ItemDrawBookState;
use crate::structs::item_draw_entry::ItemDrawEntry;
use crate::{ItemDrawKey, ItemDrawRefusal};

/// The item stack copies of the loader that mods draw with: an id with a generation and an owner mod for each, and
/// the addresses that wait to be freed.
///
/// The book makes no game call and takes no thread rule. `release`, `world_left` and `disable_owner` only move an
/// address from the live copies to the free queue; `take_frees` gives the queue to the code that runs on the game
/// thread, which destroys each game stack and gives its buffer back. So a mod may drop a handle on any thread. A
/// copy that is queued is no longer live: a later `address` or `release` of its key gives `Stale`.
///
/// Ids come from one counter that never goes back. `world_left` frees every copy and starts a new generation.
pub struct ItemDrawBook {
    state: Mutex<ItemDrawBookState>,
}

impl Default for ItemDrawBook {
    fn default() -> Self {
        Self::new()
    }
}

impl ItemDrawBook {
    /// The most live copies one mod holds: a hard bound, room for two full contents reads of a box of 27 bundles of
    /// 64 items (1755 copies each) and more.
    pub const MAX_PER_OWNER: usize = 4096;

    pub fn new() -> Self {
        Self {
            state: Mutex::new(ItemDrawBookState {
                next_id: 1,
                generation: 0,
                live: BTreeMap::new(),
                frees: Vec::new(),
                per_owner: HashMap::new(),
            }),
        }
    }

    // The state holds plain data, so a lock poisoned by a panic is recovered.
    fn lock(&self) -> MutexGuard<'_, ItemDrawBookState> {
        self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Takes the copy at `address` for the mod `owner`. `TooMany` when the mod holds `MAX_PER_OWNER` copies already;
    /// the address is then still the caller's to free.
    pub fn add(&self, owner: usize, address: usize) -> Result<ItemDrawKey, ItemDrawRefusal> {
        let mut state = self.lock();
        let count = state.per_owner.entry(owner).or_insert(0);
        if *count >= Self::MAX_PER_OWNER {
            return Err(ItemDrawRefusal::TooMany);
        }
        *count += 1;
        let id = state.next_id;
        state.next_id += 1;
        state.live.insert(id, ItemDrawEntry { owner, address });
        Ok(ItemDrawKey { id, generation: state.generation })
    }

    /// The address of the copy `key`; `Stale` for a copy of an older generation or one released.
    pub fn address(&self, key: ItemDrawKey) -> Result<usize, ItemDrawRefusal> {
        let state = self.lock();
        if key.generation != state.generation {
            return Err(ItemDrawRefusal::Stale);
        }
        state.live.get(&key.id).map(|entry| entry.address).ok_or(ItemDrawRefusal::Stale)
    }

    /// The owner of the copy `key`; `Stale` as for `address`.
    pub fn owner(&self, key: ItemDrawKey) -> Result<usize, ItemDrawRefusal> {
        let state = self.lock();
        if key.generation != state.generation {
            return Err(ItemDrawRefusal::Stale);
        }
        state.live.get(&key.id).map(|entry| entry.owner).ok_or(ItemDrawRefusal::Stale)
    }

    /// Queues the free of the copy `key` of `owner`. Allowed on any thread.
    pub fn release(&self, owner: usize, key: ItemDrawKey) -> Result<(), ItemDrawRefusal> {
        let mut state = self.lock();
        if key.generation != state.generation {
            return Err(ItemDrawRefusal::Stale);
        }
        let entry = state.live.get(&key.id).ok_or(ItemDrawRefusal::Stale)?;
        if entry.owner != owner {
            return Err(ItemDrawRefusal::NotOwner);
        }
        if let Some(entry) = state.live.remove(&key.id) {
            state.frees.push(entry.address);
            Self::forget_one(&mut state, owner);
        }
        Ok(())
    }

    /// The addresses to free now; the queue is empty after the call.
    pub fn take_frees(&self) -> Vec<usize> {
        std::mem::take(&mut self.lock().frees)
    }

    /// Whether an address waits to be freed.
    pub fn has_frees(&self) -> bool {
        !self.lock().frees.is_empty()
    }

    /// Queues the free of every live copy and starts a new generation. Gives the number of copies queued.
    pub fn world_left(&self) -> usize {
        let mut state = self.lock();
        let live = std::mem::take(&mut state.live);
        let count = live.len();
        state.frees.extend(live.into_values().map(|entry| entry.address));
        state.per_owner.clear();
        state.generation += 1;
        count
    }

    /// Queues the free of the copies of `owner`. Gives the number of copies queued.
    pub fn disable_owner(&self, owner: usize) -> usize {
        let mut state = self.lock();
        let ids: Vec<u64> = state.live.iter().filter(|(_, entry)| entry.owner == owner).map(|(id, _)| *id).collect();
        for id in &ids {
            if let Some(entry) = state.live.remove(id) {
                state.frees.push(entry.address);
            }
        }
        state.per_owner.remove(&owner);
        ids.len()
    }

    // Counts one live copy of `owner` less.
    fn forget_one(state: &mut ItemDrawBookState, owner: usize) {
        if let Some(count) = state.per_owner.get_mut(&owner) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                state.per_owner.remove(&owner);
            }
        }
    }
}
