use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::{Api, Logger};

/// The mod-side state of this DLL: the loader table, the mod logger and the disabled flag.
///
/// The SDK is linked into each mod, so every mod has its own copy.
pub struct ModRuntime {
    api: Api,
    logger: Logger,
}

static RUNTIME: OnceLock<ModRuntime> = OnceLock::new();
static DISABLED: AtomicBool = AtomicBool::new(false);

impl ModRuntime {
    /// Keeps `api` and a logger for `module`. The first call wins; later calls return the first.
    pub fn install(api: Api, module: &str) -> &'static ModRuntime {
        RUNTIME.get_or_init(|| ModRuntime { logger: Logger::new(&api, module), api })
    }

    /// The installed state, once `amml_mod_load` ran.
    pub fn get() -> Option<&'static ModRuntime> {
        RUNTIME.get()
    }

    pub fn api(&self) -> &Api {
        &self.api
    }

    pub fn logger(&self) -> &Logger {
        &self.logger
    }

    /// Whether a panic in this mod has disabled its detours.
    pub fn is_disabled() -> bool {
        DISABLED.load(Ordering::SeqCst)
    }

    /// Marks the mod disabled and returns whether it was enabled before, so only the first caller logs.
    pub fn disable() -> bool {
        !DISABLED.swap(true, Ordering::SeqCst)
    }
}
