use std::collections::{BTreeMap, HashMap};

use crate::structs::item_draw_entry::ItemDrawEntry;

/// The state behind the lock of `ItemDrawBook`.
pub(crate) struct ItemDrawBookState {
    pub(crate) next_id: u64,
    pub(crate) generation: u32,
    pub(crate) live: BTreeMap<u64, ItemDrawEntry>,
    pub(crate) frees: Vec<usize>,
    // The number of live copies of each owner, so a cap test does not walk every copy.
    pub(crate) per_owner: HashMap<usize, usize>,
}
