pub mod id;
pub mod resolved;

use std::collections::BTreeMap;

pub use id::BuildId;
pub use resolved::Resolved;

use super::EntryId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildData {
    pub build: BuildId,
    pub entries: BTreeMap<EntryId, Resolved>,
}

impl BuildData {
    pub fn new(build: BuildId) -> Self {
        Self { build, entries: BTreeMap::new() }
    }
}
