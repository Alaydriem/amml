use serde::{Deserialize, Serialize};

use super::{AmmlSection, FileRef, IndexSummary};

/// `registry.json`, the one small file a launcher fetches first.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistryRoot {
    pub schema: u32,
    pub name: String,
    pub published: String,
    pub mod_count: u32,
    pub amml: AmmlSection,
    pub index: IndexSummary,
    pub names: FileRef,
}
