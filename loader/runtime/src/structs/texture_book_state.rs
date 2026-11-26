use std::collections::BTreeMap;

use crate::TextureRelease;
use crate::structs::texture_entry::TextureEntry;

/// What `TextureBook` keeps.
pub(crate) struct TextureBookState {
    pub(crate) next_id: u64,
    pub(crate) generation: u32,
    pub(crate) next_name: u64,
    pub(crate) free_names: Vec<String>,
    pub(crate) entries: BTreeMap<u64, TextureEntry>,
    pub(crate) releases: Vec<TextureRelease>,
}
