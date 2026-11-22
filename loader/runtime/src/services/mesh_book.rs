use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard};

use crate::structs::mesh_book_state::MeshBookState;
use crate::structs::mesh_entry::MeshEntry;
use crate::{MeshFree, MeshKey, MeshKind, MeshRefusal};

/// The meshes of the loader: an id with a generation and an owner mod for each, and the game objects that the render
/// thread must destroy.
///
/// The book makes no game call. A call that ends the life of a mesh (`free`, `disable_owner`, `world_left`,
/// `device_reset`, a graphics mode switch seen by `note_vibrant`) only removes it from the book and queues its game
/// object: from then on every use of the id gives `Stale`. `take_due` gives the queue to the render thread, which
/// destroys the objects at its next `renderLevel` call; so a call is allowed from any thread, and the game never sees a
/// mesh destroyed while the render thread draws.
///
/// Ids come from one counter that never goes back, so an id is never given twice. A world leave, a render device reset
/// and a switch of the graphics mode each free every mesh and start a new generation: an id of an older generation gives
/// `Stale`.
pub struct MeshBook {
    state: Mutex<MeshBookState>,
    // True while `due` holds a mesh; the render detour reads it without the lock.
    pending: AtomicBool,
}

impl Default for MeshBook {
    fn default() -> Self {
        Self::new()
    }
}

impl MeshBook {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(MeshBookState {
                next_id: 1,
                generation: 0,
                vibrant: None,
                entries: BTreeMap::new(),
                due: Vec::new(),
            }),
            pending: AtomicBool::new(false),
        }
    }

    // The state holds plain data, so a lock poisoned by a panic is recovered.
    fn lock(&self) -> MutexGuard<'_, MeshBookState> {
        self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Adds the colour mesh at `handle` for the mod `owner` and gives its name.
    pub fn create(&self, owner: usize, handle: usize) -> MeshKey {
        self.add(owner, handle, MeshKind::Colour)
    }

    /// Adds the block mesh at `handle` for the mod `owner` and gives its name.
    pub fn create_block(&self, owner: usize, handle: usize) -> MeshKey {
        self.add(owner, handle, MeshKind::Block)
    }

    fn add(&self, owner: usize, handle: usize, kind: MeshKind) -> MeshKey {
        let mut state = self.lock();
        let id = state.next_id;
        state.next_id += 1;
        state.entries.insert(id, MeshEntry { owner, handle, kind });
        MeshKey { id, generation: state.generation }
    }

    /// The address of the game mesh `key` of `owner`.
    pub fn handle(&self, owner: usize, key: MeshKey) -> Result<usize, MeshRefusal> {
        let state = self.lock();
        let entry = Self::entry_of(&state, key)?;
        if entry.owner == owner { Ok(entry.handle) } else { Err(MeshRefusal::NotOwner) }
    }

    /// The address and the kind of the game mesh `key` of `owner`.
    pub fn entry(&self, owner: usize, key: MeshKey) -> Result<(usize, MeshKind), MeshRefusal> {
        let state = self.lock();
        let entry = Self::entry_of(&state, key)?;
        if entry.owner == owner { Ok((entry.handle, entry.kind)) } else { Err(MeshRefusal::NotOwner) }
    }

    /// Frees `key` of `owner`: the id is stale at once and the game mesh is queued for the render thread.
    pub fn free(&self, owner: usize, key: MeshKey) -> Result<(), MeshRefusal> {
        let mut state = self.lock();
        let entry = Self::entry_of(&state, key)?;
        if entry.owner != owner {
            return Err(MeshRefusal::NotOwner);
        }
        let handle = entry.handle;
        state.entries.remove(&key.id);
        state.due.push(MeshFree { key, handle });
        self.pending.store(true, Ordering::Release);
        Ok(())
    }

    /// Whether `key` is a mesh of the current generation that was not freed.
    pub fn is_current(&self, key: MeshKey) -> bool {
        Self::entry_of(&self.lock(), key).is_ok()
    }

    /// Frees every mesh of `owner`. Gives the number of meshes freed.
    pub fn disable_owner(&self, owner: usize) -> usize {
        let mut state = self.lock();
        let ids: Vec<u64> = state.entries.iter().filter(|(_, entry)| entry.owner == owner).map(|(id, _)| *id).collect();
        let generation = state.generation;
        for id in &ids {
            if let Some(entry) = state.entries.remove(id) {
                state.due.push(MeshFree { key: MeshKey { id: *id, generation }, handle: entry.handle });
            }
        }
        self.mark(&state);
        ids.len()
    }

    /// The world is left: every mesh is freed and the generation changes. Gives the number of meshes freed.
    pub fn world_left(&self) -> usize {
        self.invalidate()
    }

    /// The render device was reset or the surface lost: every mesh is freed and the generation changes. Gives the
    /// number of meshes freed.
    pub fn device_reset(&self) -> usize {
        self.invalidate()
    }

    /// Notes the graphics mode of the frame. The first call only records it; a later call with another value frees every
    /// mesh and changes the generation. True when it did.
    pub fn note_vibrant(&self, vibrant: bool) -> bool {
        {
            let mut state = self.lock();
            let previous = state.vibrant.replace(vibrant);
            if previous.is_none_or(|previous| previous == vibrant) {
                return false;
            }
        }
        self.invalidate();
        true
    }

    /// Forgets the graphics mode seen so far: the next `note_vibrant` only records, as the first one does. For a reader
    /// that stops reading the mode (no listener, no mesh), so a mode read after a long pause is not taken as a switch.
    pub fn forget_vibrant(&self) {
        self.lock().vibrant = None;
    }

    /// The game meshes to destroy now; the queue is empty after it. Called on the render thread.
    pub fn take_due(&self) -> Vec<MeshFree> {
        let mut state = self.lock();
        self.pending.store(false, Ordering::Release);
        std::mem::take(&mut state.due)
    }

    /// Whether a game mesh waits to be destroyed. One atomic load, no lock.
    pub fn has_due(&self) -> bool {
        self.pending.load(Ordering::Acquire)
    }

    /// The number of live meshes.
    pub fn len(&self) -> usize {
        self.lock().entries.len()
    }

    /// Whether the book holds no live mesh.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The current generation.
    pub fn generation(&self) -> u32 {
        self.lock().generation
    }

    fn invalidate(&self) -> usize {
        let mut state = self.lock();
        let generation = state.generation;
        let entries = std::mem::take(&mut state.entries);
        let count = entries.len();
        for (id, entry) in entries {
            state.due.push(MeshFree { key: MeshKey { id, generation }, handle: entry.handle });
        }
        state.generation = generation.wrapping_add(1);
        self.mark(&state);
        count
    }

    fn mark(&self, state: &MeshBookState) {
        if !state.due.is_empty() {
            self.pending.store(true, Ordering::Release);
        }
    }

    fn entry_of(state: &MeshBookState, key: MeshKey) -> Result<&MeshEntry, MeshRefusal> {
        if key.generation != state.generation {
            return Err(MeshRefusal::Stale);
        }
        state.entries.get(&key.id).ok_or(MeshRefusal::Stale)
    }
}
