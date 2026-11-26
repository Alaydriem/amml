use std::collections::BTreeMap;

use crate::structs::group_entry::GroupEntry;

/// What `GroupBook` keeps.
pub(crate) struct GroupBookState {
    pub(crate) next_id: u32,
    pub(crate) generation: u32,
    pub(crate) entries: BTreeMap<u32, GroupEntry>,
    pub(crate) due: Vec<usize>,
}
