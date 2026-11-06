//! Test-only fixture: dummy AMML and mod releases, a local HTTP server, and a registry tree built by the real CLI.
#![forbid(unsafe_code)]
#![allow(clippy::unwrap_used, clippy::expect_used)]

pub mod services;
pub mod structs;

pub use services::*;
pub use structs::*;
