use std::sync::{Mutex, MutexGuard};

use crate::services::LightDiff;
use crate::structs::{LightChange, LightPoint, LightRefusal};

/// The held lights of the mods: one slot per mod, at most `SLOTS`, in the order the slots were taken. Each
/// change gives the subchunks to rebuild (`LightDiff`): those where the old and the new lights differ.
pub struct LightBook {
    slots: Mutex<Vec<(usize, LightPoint)>>,
    diff: LightDiff,
}

impl Default for LightBook {
    fn default() -> Self {
        Self::new()
    }
}

impl LightBook {
    pub const SLOTS: usize = 4;

    pub fn new() -> Self {
        Self { slots: Mutex::new(Vec::new()), diff: LightDiff::new() }
    }

    // The slots hold plain data, so a lock poisoned by a panic is recovered.
    fn lock(&self) -> MutexGuard<'_, Vec<(usize, LightPoint)>> {
        self.slots.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Sets the light of `owner`; `None` clears it. An unchanged light gives an empty change. A new owner
    /// when every slot is taken gives `Full` and changes nothing. `published` gets the lights after the
    /// change, under the book's lock, so two writers never publish out of order.
    pub fn set(
        &self,
        owner: usize,
        point: Option<LightPoint>,
        published: impl FnOnce(&[LightPoint]),
    ) -> Result<LightChange, LightRefusal> {
        let mut slots = self.lock();
        let index = slots.iter().position(|(held, _)| *held == owner);
        let old = index.map(|index| slots[index].1);
        if old == point {
            return Ok(LightChange::default());
        }
        let before = Self::points_of(&slots);
        match (index, point) {
            (Some(index), Some(point)) => slots[index].1 = point,
            (Some(index), None) => {
                slots.remove(index);
            }
            (None, Some(point)) => {
                if slots.len() >= Self::SLOTS {
                    return Err(LightRefusal::Full);
                }
                slots.push((owner, point));
            }
            (None, None) => {}
        }
        let after = Self::points_of(&slots);
        published(&after);
        Ok(self.diff.change(&before, &after))
    }

    /// Clears every light; the change holds the subchunks each lit.
    pub fn clear_all(&self, published: impl FnOnce(&[LightPoint])) -> LightChange {
        let mut slots = self.lock();
        let before = Self::points_of(&slots);
        slots.clear();
        published(&[]);
        self.diff.change(&before, &[])
    }

    /// Whether `owner` holds a slot.
    pub fn holds(&self, owner: usize) -> bool {
        self.lock().iter().any(|(held, _)| *held == owner)
    }

    /// The lights in slot order.
    pub fn points(&self) -> Vec<LightPoint> {
        Self::points_of(&self.lock())
    }

    fn points_of(slots: &[(usize, LightPoint)]) -> Vec<LightPoint> {
        slots.iter().map(|(_, point)| *point).collect()
    }
}
