use serde::{Deserialize, Serialize};

use super::{Listing, ModRelease};

/// `mods/@<namespace>/<name>.json`, the source of truth for one mod.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModDocument {
    pub schema: u32,
    pub id: String,
    pub namespace: String,
    pub name: String,
    pub repo: String,
    pub core: bool,
    pub default: bool,
    pub listing: Listing,
    // newest first, at most `RegistryUpdater::MAX_MOD_VERSIONS`
    pub versions: Vec<ModRelease>,
}
