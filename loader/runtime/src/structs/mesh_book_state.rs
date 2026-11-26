use std::collections::BTreeMap;

use crate::MeshFree;
use crate::structs::mesh_entry::MeshEntry;

/// What `MeshBook` keeps.
pub(crate) struct MeshBookState {
    pub(crate) next_id: u64,
    pub(crate) generation: u32,
    /// The graphics mode the last `note_vibrant` saw; `None` before the first.
    pub(crate) vibrant: Option<bool>,
    pub(crate) entries: BTreeMap<u64, MeshEntry>,
    pub(crate) due: Vec<MeshFree>,
}
