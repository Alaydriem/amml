//! The mod-side library of AMML: a checked view of the loader function table, a logger, mod
//! folders, a panic-safe detour guard and the `amml_mod!` / `amml_detour!` macros.

#![deny(unsafe_code)]

pub mod amml_mod;
pub mod api;
pub mod detour_guard;
pub mod hook_handle;
pub mod logger;
mod macros;
pub mod mod_dirs;
pub mod mod_glue;
pub mod mod_runtime;
pub mod panic_text;
pub mod sdk_error;

pub use amml_mod::AmmlMod;
pub use api::Api;
pub use detour_guard::DetourGuard;
pub use hook_handle::HookHandle;
pub use logger::Logger;
pub use mod_dirs::ModDirs;
pub use mod_glue::ModGlue;
pub use mod_runtime::ModRuntime;
pub use panic_text::PanicText;
pub use sdk_error::SdkError;

/// Re-exported so the macros and mods name the contract types through one crate.
pub use amml_abi;
