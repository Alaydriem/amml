use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard};

use crate::structs::group_book_state::GroupBookState;
use crate::structs::group_entry::GroupEntry;
use crate::{GroupKey, GroupRefusal};

/// The mesh groups of the loader: an id with a generation and an owner mod for each, and the game buffers the render
/// thread must free.
///
/// A group is a game `MeshData` buffer the loader owns, bound to one mod and one world and usable in any dispatch of
/// that world. The book makes no game call. A call that ends the life of a group without a finish (`discard`,
/// `disable_owner`, `world_left`, `device_reset`) removes it from the book and queues its buffer: from then on every use of the id gives
/// `Stale`, and `take_due` gives the queue to the render thread, which frees the buffers at its next `renderLevel` call.
/// `finish` removes a group and hands its buffer to the caller, which owns it from then on.
///
/// Ids come from one counter that never goes back, so an id is never given twice; a generation changes at each world
/// leave and device reset.
pub struct GroupBook {
    state: Mutex<GroupBookState>,
    // True while `due` holds a buffer; the render detour reads it without the lock.
    pending: AtomicBool,
}

impl Default for GroupBook {
    fn default() -> Self {
        Self::new()
    }
}

impl GroupBook {
    /// The most open groups one mod holds.
    pub const MAX_PER_MOD: usize = 64;

    pub fn new() -> Self {
        Self {
            state: Mutex::new(GroupBookState {
                next_id: 1,
                generation: 0,
                entries: std::collections::BTreeMap::new(),
                due: Vec::new(),
            }),
            pending: AtomicBool::new(false),
        }
    }

    // The state holds plain data, so a lock poisoned by a panic is recovered.
    fn lock(&self) -> MutexGuard<'_, GroupBookState> {
        self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Opens a group of `owner` over the game buffer at `handle`. `TooMany` when `owner` holds `MAX_PER_MOD` groups.
    pub fn open(&self, owner: usize, handle: usize) -> Result<GroupKey, GroupRefusal> {
        let mut state = self.lock();
        if state.entries.values().filter(|entry| entry.owner == owner).count() >= Self::MAX_PER_MOD {
            return Err(GroupRefusal::TooMany);
        }
        let id = state.next_id;
        state.next_id = id.checked_add(1).ok_or(GroupRefusal::TooMany)?;
        state.entries.insert(id, GroupEntry { owner, handle });
        Ok(GroupKey { id, generation: state.generation })
    }

    /// The address of the buffer of the group `key` of `owner`.
    pub fn handle(&self, owner: usize, key: GroupKey) -> Result<usize, GroupRefusal> {
        let state = self.lock();
        let entry = Self::entry_of(&state, key)?;
        if entry.owner == owner { Ok(entry.handle) } else { Err(GroupRefusal::NotOwner) }
    }

    /// Ends the group `key` of `owner` and gives its buffer to the caller, which owns it from then on.
    pub fn finish(&self, owner: usize, key: GroupKey) -> Result<usize, GroupRefusal> {
        let mut state = self.lock();
        let entry = Self::entry_of(&state, key)?;
        if entry.owner != owner {
            return Err(GroupRefusal::NotOwner);
        }
        let handle = entry.handle;
        state.entries.remove(&key.id);
        Ok(handle)
    }

    /// Ends the group `key` of `owner` without a finish: the group is gone, its slot is free and its buffer is queued.
    pub fn discard(&self, owner: usize, key: GroupKey) -> Result<(), GroupRefusal> {
        let mut state = self.lock();
        let entry = Self::entry_of(&state, key)?;
        if entry.owner != owner {
            return Err(GroupRefusal::NotOwner);
        }
        let handle = entry.handle;
        state.entries.remove(&key.id);
        state.due.push(handle);
        self.mark(&state);
        Ok(())
    }

    /// Drops every group of `owner` and queues their buffers. Gives the number dropped.
    pub fn disable_owner(&self, owner: usize) -> usize {
        let mut state = self.lock();
        let ids: Vec<u32> = state.entries.iter().filter(|(_, entry)| entry.owner == owner).map(|(id, _)| *id).collect();
        for id in &ids {
            if let Some(entry) = state.entries.remove(id) {
                state.due.push(entry.handle);
            }
        }
        self.mark(&state);
        ids.len()
    }

    /// The world is left: every group is dropped, its buffer queued, and the generation changes. Gives the number dropped.
    pub fn world_left(&self) -> usize {
        self.invalidate()
    }

    /// The render device was reset or the surface lost: as `world_left`.
    pub fn device_reset(&self) -> usize {
        self.invalidate()
    }

    /// The buffers to free now; the queue is empty after it. Called on the render thread.
    pub fn take_due(&self) -> Vec<usize> {
        let mut state = self.lock();
        self.pending.store(false, Ordering::Release);
        std::mem::take(&mut state.due)
    }

    /// Whether a buffer waits to be freed. One atomic load, no lock.
    pub fn has_due(&self) -> bool {
        self.pending.load(Ordering::Acquire)
    }

    /// The number of open groups.
    pub fn len(&self) -> usize {
        self.lock().entries.len()
    }

    /// Whether the book holds no open group.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn invalidate(&self) -> usize {
        let mut state = self.lock();
        let entries = std::mem::take(&mut state.entries);
        let count = entries.len();
        state.due.extend(entries.into_values().map(|entry| entry.handle));
        state.generation = state.generation.wrapping_add(1);
        self.mark(&state);
        count
    }

    fn mark(&self, state: &GroupBookState) {
        if !state.due.is_empty() {
            self.pending.store(true, Ordering::Release);
        }
    }

    fn entry_of(state: &GroupBookState, key: GroupKey) -> Result<&GroupEntry, GroupRefusal> {
        if key.generation != state.generation {
            return Err(GroupRefusal::Stale);
        }
        state.entries.get(&key.id).ok_or(GroupRefusal::Stale)
    }
}
