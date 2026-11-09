pub mod fingerprint;
pub mod id;
pub mod resolved;

use std::collections::BTreeMap;

pub use fingerprint::Fingerprint;
pub use id::BuildId;
pub use resolved::Resolved;

use super::EntryId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildData {
    pub build: BuildId,
    pub entries: BTreeMap<EntryId, Resolved>,
    // Fingerprint of every resolved or failed entry that has one.
    pub fingerprints: BTreeMap<EntryId, Fingerprint>,
    // Entries that did not resolve, with the failure reason text. Information only.
    pub failed: BTreeMap<EntryId, String>,
}

impl BuildData {
    pub fn new(build: BuildId) -> Self {
        Self { build, entries: BTreeMap::new(), fingerprints: BTreeMap::new(), failed: BTreeMap::new() }
    }
}
