use std::collections::BTreeMap;

use crate::GhostEntity;
use crate::structs::ghost_entity_entry::GhostEntityEntry;

/// What `GhostEntityBook` keeps.
pub(crate) struct GhostEntityBookState {
    pub(crate) next_id: u32,
    pub(crate) generation: u32,
    pub(crate) entries: BTreeMap<u32, GhostEntityEntry>,
    pub(crate) due: Vec<GhostEntity>,
}
