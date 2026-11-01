use serde::Deserialize;

use super::TokenEntry;

/// The Paper `get_tokens` export in JSON form: `{"tokens": [...]}`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct TokenFile {
    pub tokens: Vec<TokenEntry>,
}
