#![allow(clippy::unwrap_used)]

mod api_table;
mod boundary;
mod entry_point;
mod folder_redirect;
#[cfg(feature = "folder-trace")]
mod folder_trace;
mod game_heap;
mod hook_engine;
mod host_paths;
mod host_startup;
mod import_hook;
mod instance_settings;
mod loader_log;
mod loader_settings;
mod mod_loader;
mod standalone_host;
mod startup_gate;
mod support;
mod version_label;
