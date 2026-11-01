use serde::Deserialize;

use super::Owner;

/// A selector enum: code that picks a token from data matches on its variants.
///
/// Each variant pairs a variant name with a token name, for example `("Lime", "--color-scene-1")`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SelectorSpec {
    #[serde(rename = "crate")]
    pub crate_owner: Owner,
    pub name: String,
    pub variants: Vec<(String, String)>,
}
