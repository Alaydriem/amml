use serde::Deserialize;

use super::Listing;

/// The part of a mod amml.hcl that the registry reads. Other attributes belong to the loader.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ModManifest {
    #[serde(default)]
    pub namespace: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub api_version: u32,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub amml_version: Option<String>,
    #[serde(default)]
    pub listing: Option<Listing>,
}
