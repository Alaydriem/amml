use crate::structs::{Held, ResourceError, ResourceId, ResourceKind, Slot};

const SLOT_MASK: u64 = 0xFFFF_FFFF;
pub(crate) const GENERATION_MASK: u32 = 0x00FF_FFFF;

/// The slots and free list of a resource store, behind its lock, and the highest epoch taken back.
pub(crate) struct StoreState<T> {
    pub(crate) slots: Vec<Slot<T>>,
    pub(crate) free: Vec<u32>,
    pub(crate) retired_up_to: u64,
}

impl<T> StoreState<T> {
    pub(crate) fn new() -> Self {
        Self { slots: Vec::new(), free: Vec::new(), retired_up_to: 0 }
    }

    pub(crate) fn id_of(slot: usize, generation: u32, kind: ResourceKind) -> ResourceId {
        ResourceId((slot as u64) | (u64::from(generation) << 32) | (u64::from(kind.0) << 56))
    }

    // Slot of a live resource whose slot, generation and kind byte all match the id.
    pub(crate) fn find(&self, id: ResourceId) -> Result<usize, ResourceError> {
        let slot = (id.0 & SLOT_MASK) as usize;
        let generation = ((id.0 >> 32) as u32) & GENERATION_MASK;
        let kind = (id.0 >> 56) as u8;
        match self.slots.get(slot) {
            Some(Slot { generation: current, held: Some(held) }) if *current == generation && held.kind.0 == kind => {
                Ok(slot)
            }
            _ => Err(ResourceError::Stale),
        }
    }

    // Empties the slot and raises its generation. A slot whose generation
    // would pass 24 bits is retired: it never goes back on the free list, so
    // an old id cannot become valid again.
    pub(crate) fn release(&mut self, slot: usize) -> Option<Held<T>> {
        let entry = self.slots.get_mut(slot)?;
        let held = entry.held.take()?;
        if entry.generation < GENERATION_MASK {
            entry.generation += 1;
            self.free.push(slot as u32);
        }
        Some(held)
    }

    pub(crate) fn take_where(&mut self, mut matches: impl FnMut(&Held<T>) -> bool) -> Vec<Held<T>> {
        let hit: Vec<usize> = self
            .slots
            .iter()
            .enumerate()
            .filter(|(_, entry)| entry.held.as_ref().is_some_and(&mut matches))
            .map(|(slot, _)| slot)
            .collect();
        hit.into_iter().filter_map(|slot| self.release(slot)).collect()
    }
}
