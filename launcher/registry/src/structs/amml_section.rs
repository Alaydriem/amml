use serde::{Deserialize, Serialize};

use super::AmmlRelease;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AmmlSection {
    pub repo: String,
    // newest first, at most `RegistryUpdater::MAX_AMML_VERSIONS`
    pub versions: Vec<AmmlRelease>,
}
