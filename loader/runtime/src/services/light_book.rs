use std::sync::{Mutex, MutexGuard};

use crate::structs::{LightChange, LightPoint, LightRefusal};

/// The held lights of the mods: one slot per mod, at most `SLOTS`, in the order the slots were taken. Each
/// change gives the boxes to rebuild: the reach of the old and of the new light of that mod.
pub struct LightBook {
    slots: Mutex<Vec<(usize, LightPoint)>>,
}

impl Default for LightBook {
    fn default() -> Self {
        Self::new()
    }
}

impl LightBook {
    pub const SLOTS: usize = 4;

    pub fn new() -> Self {
        Self { slots: Mutex::new(Vec::new()) }
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
        published(&Self::points_of(&slots));
        let mut change = LightChange::default();
        for light in [old, point].into_iter().flatten() {
            change.add(light.reach_box());
        }
        Ok(change)
    }

    /// Clears every light; the change holds the reach of each.
    pub fn clear_all(&self, published: impl FnOnce(&[LightPoint])) -> LightChange {
        let mut slots = self.lock();
        let mut change = LightChange::default();
        for (_, light) in slots.drain(..) {
            change.add(light.reach_box());
        }
        published(&[]);
        change
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
