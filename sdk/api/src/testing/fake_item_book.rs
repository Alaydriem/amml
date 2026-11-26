use std::sync::{Arc, Mutex, MutexGuard};

use amml_abi::{AmmlContainerEntry, AmmlItemDraw, AmmlStatus};

use crate::testing::{FakeEntry, FakeItemBookData};

// The book of the newest fake. A thread that holds no fake reaches it here: the loader checks a release at once on
// any thread and defers only the free, and so does the fake.
static ACTIVE: Mutex<Option<Arc<Mutex<FakeItemBookData>>>> = Mutex::new(None);

/// The item draw side of a `FakeGame`: the draws of the current generation with the contents lists that gave them, as
/// the loader's book keeps them.
pub(crate) struct FakeItemBook {
    data: Arc<Mutex<FakeItemBookData>>,
}

impl Default for FakeItemBook {
    fn default() -> Self {
        let data = Arc::new(Mutex::new(FakeItemBookData::default()));
        *ACTIVE.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(data.clone());
        Self { data }
    }
}

impl FakeItemBook {
    fn lock(&self) -> MutexGuard<'_, FakeItemBookData> {
        self.data.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Opens the contents `entries`: the list id and the number of entries. `REFUSED` when the entries of the top level
    /// alone pass the 4096 draws of a mod or the 1755 draws of one read, with nothing left behind.
    pub(crate) fn open(&self, module: &str, entries: &[FakeEntry]) -> Result<(u64, u32), AmmlStatus> {
        let mut data = self.lock();
        let mut made = Vec::new();
        let mut opened = 0usize;
        match data.build(module, entries, &mut made, &mut opened) {
            Ok(slots) => {
                let len = u32::try_from(slots.len()).unwrap_or(u32::MAX);
                data.next_list += 1;
                let id = data.next_list;
                data.lists.insert(id, (module.to_string(), slots));
                Ok((id, len))
            }
            Err(status) => {
                for id in made {
                    data.live.remove(&id);
                }
                Err(status)
            }
        }
    }

    /// Entry `index` of the list `contents`, which the caller then owns. `INVALID_ARGUMENT` past the end, `REFUSED` for an
    /// entry read before, `STALE` for a list that is closed.
    pub(crate) fn entry(&self, module: &str, contents: u64, index: u32) -> Result<AmmlContainerEntry, AmmlStatus> {
        let mut data = self.lock();
        let (owner, slots) = data.lists.get_mut(&contents).ok_or(AmmlStatus::STALE)?;
        if owner != module {
            return Err(AmmlStatus::REFUSED);
        }
        let slot = slots.get_mut(index as usize).ok_or(AmmlStatus::INVALID_ARGUMENT)?;
        if slot.taken {
            return Err(AmmlStatus::REFUSED);
        }
        slot.taken = true;
        Ok(AmmlContainerEntry {
            slot: slot.entry.slot,
            has_bundle_weight: u8::from(slot.entry.bundle_weight.is_some()),
            bundle_weight: slot.entry.bundle_weight.unwrap_or(0),
            item: slot.entry.item.raw_snapshot(),
            draw: slot.draw,
            bundle: slot.nested,
            ..AmmlContainerEntry::empty()
        })
    }

    /// The type name of entry `index` of the list `contents`.
    pub(crate) fn name(&self, module: &str, contents: u64, index: u32) -> Result<&'static str, AmmlStatus> {
        let data = self.lock();
        let (owner, slots) = data.lists.get(&contents).ok_or(AmmlStatus::STALE)?;
        if owner != module {
            return Err(AmmlStatus::REFUSED);
        }
        slots.get(index as usize).map(|slot| slot.entry.item.type_name).ok_or(AmmlStatus::INVALID_ARGUMENT)
    }

    /// Closes the list `contents`: the draws nobody took are freed.
    pub(crate) fn close(&self, module: &str, contents: u64) {
        let mut data = self.lock();
        if data.lists.get(&contents).is_some_and(|(owner, _)| owner != module) {
            return;
        }
        data.close(contents);
    }

    /// The type name of the live draw `draw`; `STALE` for one of an older generation or released.
    pub(crate) fn name_of(&self, draw: AmmlItemDraw) -> Result<&'static str, AmmlStatus> {
        let data = self.lock();
        if !data.is_current(draw) {
            return Err(AmmlStatus::STALE);
        }
        data.live.get(&draw.id).copied().ok_or(AmmlStatus::STALE)
    }

    /// Frees `draw` at once.
    pub(crate) fn release(&self, draw: AmmlItemDraw) -> Result<(), AmmlStatus> {
        self.lock().take(draw)
    }

    /// The release of `draw` on a thread that holds no fake: checked at once (`STALE` for a draw that is not live, also
    /// one already queued), counted as held until `pump`.
    pub(crate) fn release_off_thread(draw: AmmlItemDraw) -> Result<(), AmmlStatus> {
        let active = ACTIVE.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).clone();
        let Some(active) = active else { return Err(AmmlStatus::FAILED) };
        let mut data = active.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        data.take(draw)?;
        data.queued.push(draw.id);
        Ok(())
    }

    /// Applies the queued releases; gives the number of draws freed.
    pub(crate) fn pump(&self) -> usize {
        let mut data = self.lock();
        let count = data.queued.len();
        data.queued.clear();
        count
    }

    /// The draws not freed yet: the live ones and the queued ones.
    pub(crate) fn held(&self) -> usize {
        self.lock().held()
    }

    /// Every draw is freed, every list is closed and the generation goes up.
    pub(crate) fn leave_world(&self) {
        let mut data = self.lock();
        data.live.clear();
        data.queued.clear();
        data.lists.clear();
        data.generation += 1;
    }
}
