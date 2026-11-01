use serde::Deserialize;

use super::TokenKind;

/// One Paper token, as exported: its CSS name, its type, its raw value and its description.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct TokenEntry {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: TokenKind,
    pub value: String,
    #[serde(default)]
    pub description: Option<String>,
}
