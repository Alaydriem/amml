use std::sync::{Mutex, MutexGuard};

use crate::structs::{Held, ResourceError, ResourceId, ResourceKind, Slot, StoreState, WorldEpoch};

/// Owns the resources a mod holds by id: meshes, textures, key holds.
///
/// An id carries a slot, a generation and a kind. Every remove raises the
/// slot's generation, so an id that was freed, or whose owner, world or kind
/// was taken back, gives `Stale` even after its slot is reused. The store only
/// forgets: the caller frees what `take_*` returns on the right thread.
pub struct ResourceStore<T> {
    state: Mutex<StoreState<T>>,
}

impl<T> Default for ResourceStore<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> ResourceStore<T> {
    pub fn new() -> Self {
        Self { state: Mutex::new(StoreState::new()) }
    }

    fn lock(&self) -> MutexGuard<'_, StoreState<T>> {
        self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Stores `value` for `owner`. Epoch 0 binds it to no world.
    ///
    /// An epoch at or below the highest one `take_epoch` took gives `Stale` with `value` back, for
    /// the caller to free on the right thread: that world has ended, and nothing would take the
    /// resource back again.
    pub fn insert(
        &self,
        kind: ResourceKind,
        owner: usize,
        epoch: WorldEpoch,
        value: T,
    ) -> Result<ResourceId, (ResourceError, T)> {
        let mut state = self.lock();
        if epoch.0 != 0 && epoch.0 <= state.retired_up_to {
            return Err((ResourceError::Stale, value));
        }
        let held = Held { kind, owner, epoch, value };
        if let Some(slot) = state.free.pop() {
            let entry = &mut state.slots[slot as usize];
            entry.held = Some(held);
            return Ok(StoreState::<T>::id_of(slot as usize, entry.generation, kind));
        }
        let slot = state.slots.len();
        // Generation 0 is never handed out, so a zeroed id is always stale.
        state.slots.push(Slot { generation: 1, held: Some(held) });
        Ok(StoreState::<T>::id_of(slot, 1, kind))
    }

    pub fn get(&self, id: ResourceId) -> Result<T, ResourceError>
    where
        T: Clone,
    {
        let state = self.lock();
        let slot = state.find(id)?;
        state.slots[slot].held.as_ref().map(|held| held.value.clone()).ok_or(ResourceError::Stale)
    }

    /// Like `get`, and refuses an id of another kind.
    pub fn get_kind(&self, id: ResourceId, kind: ResourceKind) -> Result<T, ResourceError>
    where
        T: Clone,
    {
        let state = self.lock();
        let slot = state.find(id)?;
        let held = state.slots[slot].held.as_ref().ok_or(ResourceError::Stale)?;
        if held.kind != kind {
            return Err(ResourceError::WrongKind);
        }
        Ok(held.value.clone())
    }

    pub fn remove(&self, id: ResourceId) -> Result<T, ResourceError> {
        let mut state = self.lock();
        let slot = state.find(id)?;
        state.release(slot).map(|held| held.value).ok_or(ResourceError::Stale)
    }

    /// Forgets every resource of `owner`.
    pub fn take_owner(&self, owner: usize) -> Vec<(ResourceKind, T)> {
        let taken = self.lock().take_where(|held| held.owner == owner);
        taken.into_iter().map(|held| (held.kind, held.value)).collect()
    }

    /// Forgets every resource bound to `epoch` and refuses later inserts for it and every earlier
    /// epoch (epochs only grow). Epoch 0 takes nothing.
    pub fn take_epoch(&self, epoch: WorldEpoch) -> Vec<(ResourceKind, T)> {
        if epoch.0 == 0 {
            return Vec::new();
        }
        let mut state = self.lock();
        state.retired_up_to = state.retired_up_to.max(epoch.0);
        let taken = state.take_where(|held| held.epoch == epoch);
        taken.into_iter().map(|held| (held.kind, held.value)).collect()
    }

    /// Forgets every resource of `kind`, for a render device reset.
    pub fn take_kind(&self, kind: ResourceKind) -> Vec<T> {
        let taken = self.lock().take_where(|held| held.kind == kind);
        taken.into_iter().map(|held| held.value).collect()
    }
}
