//! The versioned `#[repr(C)]` contract between the AMML loader and AMML mods.
#![deny(unsafe_code)]

pub mod amml_api;
pub mod amml_exports;
pub mod amml_log_level;
pub mod amml_mod_info;
pub mod amml_status;
pub mod amml_str;
pub mod known_log_level;
pub mod mod_id;

pub use amml_api::{AMML_API_VERSION, AmmlApi, HeapDeleteFn, HeapNewFn, HookFn, LogFn, ModDirFn, ResolveFn, UnhookFn};
pub use amml_exports::{AmmlExports, ModDisableFn, ModEnableFn, ModLoadFn, ModUnloadFn};
pub use amml_log_level::AmmlLogLevel;
pub use amml_mod_info::AmmlModInfo;
pub use amml_status::AmmlStatus;
pub use amml_str::AmmlStr;
pub use known_log_level::KnownLogLevel;
pub use mod_id::ModId;
