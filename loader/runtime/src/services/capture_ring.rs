use std::sync::{Mutex, MutexGuard};

use crate::structs::{CaptureSlotState, SlotRelease};

struct RingState {
    slots: Vec<CaptureSlotState>,
    every_nth: u64,
    next_order: u64,
    skipped: u64,
    closed: bool,
}

/// The slots of a capture stream: the render thread reserves one on every nth present, fills it, and the consumer
/// takes frames oldest first. No call waits: a present with no free slot is skipped and counted.
///
/// A slot goes Free, Writing (`wants`), Filled (`filled`), Taken (`take`), Free (`give_back`). A copy that does not
/// complete (a `Map` never ready, a staging texture dropped, a lost device, a failed copy) gives its Writing slot back
/// with `abandon`. `close` frees the Free and Filled slots at once. A Taken slot is freed by its `give_back`, a
/// Writing slot by its `filled` (which then gives `None` so the caller drops the copy) or its `abandon`; both of
/// those tell the caller when the buffer is to be freed now (`SlotRelease::FreeNow`).
pub struct CaptureRing {
    state: Mutex<RingState>,
}

impl CaptureRing {
    /// A `ring` of 0 gives a ring that never wants a frame; an `every_nth_present` of 0 counts as 1.
    pub fn new(ring: u8, every_nth_present: u32) -> Self {
        Self {
            state: Mutex::new(RingState {
                slots: vec![CaptureSlotState::Free; usize::from(ring)],
                every_nth: u64::from(every_nth_present.max(1)),
                next_order: 0,
                skipped: 0,
                closed: false,
            }),
        }
    }

    fn lock(&self) -> MutexGuard<'_, RingState> {
        self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Whether this present is copied: an nth present with a free slot, which is reserved (Writing). An nth present
    /// with no free slot counts as skipped. A closed ring wants nothing and counts nothing.
    pub fn wants(&self, present_index: u64) -> bool {
        let mut state = self.lock();
        if state.closed || !present_index.is_multiple_of(state.every_nth) {
            return false;
        }
        match state.slots.iter_mut().find(|slot| **slot == CaptureSlotState::Free) {
            Some(slot) => {
                *slot = CaptureSlotState::Writing { present: present_index };
                true
            }
            None => {
                state.skipped += 1;
                false
            }
        }
    }

    /// The slot `wants` reserved for this present, to copy into. No state change.
    pub fn writing(&self, present_index: u64) -> Option<usize> {
        let state = self.lock();
        state.slots.iter().position(|slot| *slot == CaptureSlotState::Writing { present: present_index })
    }

    /// Marks the copy of this present as written and gives its slot. After `close` the slot is freed and the result
    /// is `None`. `None` also for a present with no reserved slot.
    pub fn filled(&self, present_index: u64) -> Option<usize> {
        let mut state = self.lock();
        let at = state.slots.iter().position(|slot| *slot == CaptureSlotState::Writing { present: present_index })?;
        if state.closed {
            state.slots[at] = CaptureSlotState::Free;
            return None;
        }
        let order = state.next_order;
        state.next_order += 1;
        state.slots[at] = CaptureSlotState::Filled { present: present_index, order };
        Some(at)
    }

    /// Gives back the slot `wants` reserved for this present when its copy will not complete. `Returned` in an open
    /// ring (the slot is Free again), `FreeNow` after `close` (the caller frees the slot's buffer), `NotHeld` when no
    /// slot is reserved for this present.
    pub fn abandon(&self, present_index: u64) -> SlotRelease {
        self.release(CaptureSlotState::Writing { present: present_index })
    }

    /// Takes the oldest Filled slot; the consumer holds it until `give_back`. Gives its slot and present index.
    pub fn take(&self) -> Option<(usize, u64)> {
        let mut state = self.lock();
        let (at, present) = state
            .slots
            .iter()
            .enumerate()
            .filter_map(|(at, slot)| match slot {
                CaptureSlotState::Filled { present, order } => Some((*order, at, *present)),
                _ => None,
            })
            .min()
            .map(|(_, at, present)| (at, present))?;
        state.slots[at] = CaptureSlotState::Taken { present };
        Some((at, present))
    }

    /// Frees the Taken slot of this present. `Returned` in an open ring (the render thread writes into the slot's
    /// buffer again), `FreeNow` after `close` (the caller frees the buffer), `NotHeld` when no slot is taken for it.
    pub fn give_back(&self, present_index: u64) -> SlotRelease {
        self.release(CaptureSlotState::Taken { present: present_index })
    }

    // Moves the slot in state `held` to Free and says what becomes of its buffer.
    fn release(&self, held: CaptureSlotState) -> SlotRelease {
        let mut state = self.lock();
        let closed = state.closed;
        match state.slots.iter_mut().find(|slot| **slot == held) {
            Some(slot) => {
                *slot = CaptureSlotState::Free;
                if closed { SlotRelease::FreeNow } else { SlotRelease::Returned }
            }
            None => SlotRelease::NotHeld,
        }
    }

    /// Whether `close` ran.
    pub fn is_closed(&self) -> bool {
        self.lock().closed
    }

    /// Stops the ring and gives the slots to free now: the Free and Filled ones. Closing twice gives none.
    pub fn close(&self) -> Vec<usize> {
        let mut state = self.lock();
        if state.closed {
            return Vec::new();
        }
        state.closed = true;
        let mut now = Vec::new();
        for (at, slot) in state.slots.iter_mut().enumerate() {
            if matches!(slot, CaptureSlotState::Free | CaptureSlotState::Filled { .. }) {
                *slot = CaptureSlotState::Free;
                now.push(at);
            }
        }
        now
    }

    pub fn free_slots(&self) -> u32 {
        let free = self.lock().slots.iter().filter(|slot| **slot == CaptureSlotState::Free).count();
        u32::try_from(free).unwrap_or(u32::MAX)
    }

    /// The nth presents skipped because no slot was free.
    pub fn skipped(&self) -> u64 {
        self.lock().skipped
    }
}
