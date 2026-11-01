use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use amml_abi::KnownLogLevel;
use curia::{Dispatcher, InstallError, Level, Logger, SinkError};

use crate::LogRouter;

static INSTALLED: AtomicBool = AtomicBool::new(false);

/// The loader logging: one process-wide curia sink writing `loader.log` and one file per mod.
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

    /// Writes a line for `module` to `<module>.log`.
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
