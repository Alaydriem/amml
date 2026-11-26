use std::collections::{BTreeMap, VecDeque};

use crate::GlyphPage;
use crate::structs::inline_icon_entry::InlineIconEntry;

/// What `InlineIconBook` keeps under its lock.
pub(crate) struct InlineIconState {
    pub(crate) generation: u32,
    pub(crate) next_id: u64,
    pub(crate) icons: BTreeMap<u64, InlineIconEntry>,
    // Whether a cell was handed out at least once: such a cell is reused only when no unused cell is left.
    pub(crate) used: Vec<bool>,
    // The cells that were released, the oldest first.
    pub(crate) released: VecDeque<usize>,
    pub(crate) page: GlyphPage,
}
