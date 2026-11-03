use crate::{ModId, ModSource};
use amml_registry::Listing;
use std::path::PathBuf;

// A mod folder found in an instance. A mod built from source has no version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledMod {
    pub id: ModId,
    pub version: Option<String>,
    pub enabled: bool,
    pub listing: Option<Listing>,
    // 0 when the mod's `amml.hcl` could not be read
    pub api_version: u32,
    pub source: ModSource,
    pub folder: PathBuf,
}
