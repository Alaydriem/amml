use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};

use amml_abi::KnownLogLevel;
use curia::{Dispatcher, InstallError, Level, Logger, SinkError};

use crate::LogRouter;

static INSTALLED: AtomicBool = AtomicBool::new(false);
static MODS: OnceLock<Mutex<HashMap<String, (String, String)>>> = OnceLock::new();

/// The loader logging: one process-wide curia sink writing `loader.log` and one file per registered mod.
pub struct LoaderLog;

impl LoaderLog {
    /// Installs the sink. It installs once per process: a later call, with any folder, opens no
    /// files and keeps the first sink. When something else already installed a curia sink, that
    /// sink stays and this is not an error.
    pub fn install(dir: &Path) -> Result<(), SinkError> {
        if INSTALLED.swap(true, Ordering::SeqCst) {
            return Ok(());
        }
        let result = LogRouter::new(dir.to_path_buf(), Level::Debug).map(|router| {
            match Logger::install(Box::new(Dispatcher::new(vec![router]))) {
                Ok(()) | Err(InstallError::AlreadyInstalled) => (),
            }
        });
        if result.is_err() {
            INSTALLED.store(false, Ordering::SeqCst);
        }
        result
    }

    /// Makes `id` a module with its own log file, `<logs>/@<namespace>/<name>.log`. A module string that
    /// was never registered writes to `loader.log`.
    pub fn register_mod(id: &str, namespace: &str, name: &str) {
        let mods = MODS.get_or_init(|| Mutex::new(HashMap::new()));
        if let Ok(mut mods) = mods.lock() {
            mods.entry(id.to_ascii_lowercase()).or_insert_with(|| (namespace.to_string(), name.to_string()));
        }
    }

    /// The `(namespace, name)` registered for `module`, compared case-insensitively.
    pub(crate) fn registered(module: &str) -> Option<(String, String)> {
        let mods = MODS.get()?.lock().ok()?;
        mods.get(&module.to_ascii_lowercase()).cloned()
    }

    /// Writes a line for `module` to its mod log; an unregistered module writes to `loader.log`.
    pub fn module_line(module: &str, level: KnownLogLevel, message: &str) {
        match level {
            KnownLogLevel::Trace => curia::trace!(message, { module: module }),
            KnownLogLevel::Debug => curia::debug!(message, { module: module }),
            KnownLogLevel::Info => curia::info!(message, { module: module }),
            KnownLogLevel::Warn => curia::warn!(message, { module: module }),
            KnownLogLevel::Error => curia::error!(message, { module: module }),
        }
    }
}
