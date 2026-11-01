//! Translation runtime for AMML mods: compiled language packs, locale negotiation and
//! enum-keyed message lookup. Lifted from bvc; no UI dependency.

pub mod localize;
pub mod message;
pub mod negotiate;
pub mod pack;
pub mod service;
pub mod text;
