use std::collections::{BTreeMap, VecDeque};

use amml_abi::{AmmlInlineIcon, AmmlStatus};

pub(crate) struct FakeIconData {
    pub(crate) generation: u32,
    pub(crate) next_id: u64,
    // The icons by id, with their cell.
    pub(crate) icons: BTreeMap<u64, usize>,
    pub(crate) used: Vec<bool>,
    pub(crate) released: VecDeque<usize>,
    pub(crate) bitmap: bool,
    // The icons freed on a thread with no fake: checked at once, freed by `pump`.
    pub(crate) queued: Vec<AmmlInlineIcon>,
}

impl FakeIconData {
    // The cell of `icon` when it is current.
    pub(crate) fn current(&self, icon: AmmlInlineIcon) -> Result<usize, AmmlStatus> {
        if icon.generation != self.generation {
            return Err(AmmlStatus::STALE);
        }
        self.icons.get(&icon.id).copied().ok_or(AmmlStatus::STALE)
    }

    pub(crate) fn take(&mut self, icon: AmmlInlineIcon) -> Result<(), AmmlStatus> {
        self.current(icon)?;
        Ok(())
    }

    pub(crate) fn free(&mut self, icon: AmmlInlineIcon) {
        if let Some(cell) = self.icons.remove(&icon.id) {
            self.released.push_back(cell);
        }
    }
}
