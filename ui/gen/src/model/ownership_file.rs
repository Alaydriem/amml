use std::collections::BTreeMap;

use serde::Deserialize;

use super::{Owner, SelectorSpec};

/// `ownership.json`: the crate that owns each token, and the selector enums to generate.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct OwnershipFile {
    pub owners: BTreeMap<String, Owner>,
    #[serde(default)]
    pub selectors: Vec<SelectorSpec>,
}
