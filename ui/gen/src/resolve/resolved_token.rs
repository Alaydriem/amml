use crate::model::{Owner, TokenEntry};
use crate::value::TokenValue;

/// A checked token: its Paper entry, its owning crate and its parsed value. An alias stays
/// `TokenValue::Alias`; the resolver has checked that its target exists, has the same type and
/// may be used from this token's crate.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedToken {
    pub entry: TokenEntry,
    pub owner: Owner,
    pub value: TokenValue,
}
