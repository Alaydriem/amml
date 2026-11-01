//! Build-time translation tools: writes .pot files from message enums, compiles .po files
//! into language packs, makes the `en_XA` pseudo-locale, and reports coverage.
//! Ported from the bvc TypeScript tools.

pub mod app;
pub mod catalog;
pub mod error;
pub mod pack;
pub mod plural;
pub mod po;
pub mod pot;
pub mod pseudo;
pub mod report;
