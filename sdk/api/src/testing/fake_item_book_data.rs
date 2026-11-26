use std::collections::HashMap;

use amml_abi::{AmmlItemDraw, AmmlStatus};

use crate::testing::{FakeEntry, FakeItemSlot};

#[derive(Default)]
pub(crate) struct FakeItemBookData {
    pub(crate) generation: u32,
    pub(crate) next_draw: u64,
    pub(crate) next_list: u64,
    // The live draws: id and the type name of the stack.
    pub(crate) live: HashMap<u64, &'static str>,
    pub(crate) lists: HashMap<u64, (String, Vec<FakeItemSlot>)>,
    // Released on a thread with no fake: checked at once, freed by `pump`.
    pub(crate) queued: Vec<u64>,
}

impl FakeItemBookData {
    // The most draws one mod holds, as the loader's book.
    pub(crate) const MAX_DRAWS: usize = 4096;
    // The most draws one contents read makes, as the loader: 27 slots and 64 items for each as a bundle.
    pub(crate) const MAX_PER_OPEN: usize = 27 + 27 * 64;

    pub(crate) fn held(&self) -> usize {
        self.live.len() + self.queued.len()
    }

    pub(crate) fn add(&mut self, entry: &FakeEntry, opened: &mut usize) -> Result<AmmlItemDraw, AmmlStatus> {
        if self.held() >= Self::MAX_DRAWS || *opened >= Self::MAX_PER_OPEN {
            return Err(AmmlStatus::REFUSED);
        }
        *opened += 1;
        self.next_draw += 1;
        self.live.insert(self.next_draw, entry.item.type_name);
        Ok(AmmlItemDraw { id: self.next_draw, generation: self.generation, _reserved: 0 })
    }

    // The slots of `entries`, with the draws and the nested lists they need. The entries of the list come first, so a
    // full cap costs only the contents of the last bundles. An error (the list itself does not fit) leaves its draws in
    // `made`.
    pub(crate) fn build(
        &mut self,
        module: &str,
        entries: &[FakeEntry],
        made: &mut Vec<u64>,
        opened: &mut usize,
    ) -> Result<Vec<FakeItemSlot>, AmmlStatus> {
        let mut slots = Vec::new();
        for entry in entries {
            let draw = self.add(entry, opened)?;
            made.push(draw.id);
            slots.push(FakeItemSlot { entry: *entry, draw, nested: 0, taken: false });
        }
        for slot in &mut slots {
            let Some(contents) = slot.entry.bundle else { continue };
            let mut inner_made = Vec::new();
            match self.build(module, contents, &mut inner_made, opened) {
                Ok(inner) => {
                    made.extend(inner_made);
                    self.next_list += 1;
                    slot.nested = self.next_list;
                    self.lists.insert(slot.nested, (module.to_string(), inner));
                }
                Err(_) => {
                    *opened -= inner_made.len();
                    for id in inner_made {
                        self.live.remove(&id);
                    }
                }
            }
        }
        Ok(slots)
    }

    pub(crate) fn is_current(&self, draw: AmmlItemDraw) -> bool {
        draw.generation == self.generation && self.live.contains_key(&draw.id)
    }

    // Takes the draw out of the live ones at once, as the loader's book does.
    pub(crate) fn take(&mut self, draw: AmmlItemDraw) -> Result<(), AmmlStatus> {
        if !self.is_current(draw) {
            return Err(AmmlStatus::STALE);
        }
        self.live.remove(&draw.id);
        Ok(())
    }

    pub(crate) fn close(&mut self, contents: u64) {
        let Some((_, slots)) = self.lists.remove(&contents) else { return };
        for slot in slots.iter().filter(|slot| !slot.taken) {
            let _ = self.take(slot.draw);
            if slot.nested != 0 {
                self.close(slot.nested);
            }
        }
    }
}
