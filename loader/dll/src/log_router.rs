use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::LoaderLog;
use amml_runtime::ModRegistry;
use curia::{
    DEFAULT_FILE_OPEN_STRATEGY, DEFAULT_TIMEZONE_STRATEGY, FileSink, Level, LineFormatter, LogEvent, RotationStrategy,
    Sink, SinkError,
};

/// Sends each log line to a file: lines with a string field `module` that names a mod registered
/// with `LoaderLog::register_mod` go to `<dir>/@<namespace>/<name>.log`, every other line goes to
/// `loader.log`. No path is built from an unregistered module string.
///
/// A module file is opened on its first line. At most `ModRegistry::MAX_MODS` module files exist;
/// a module past that writes to `loader.log`.
pub struct LogRouter {
    dir: PathBuf,
    level: Level,
    loader: FileSink,
    modules: Mutex<HashMap<String, FileSink>>,
}

impl LogRouter {
    // curia appends `.log` to the name
    pub const LOADER_FILE: &'static str = "loader";
    const MAX_FILE_BYTES: u64 = 1_000_000;
    const ARCHIVES_KEPT: usize = 3;

    pub fn new(dir: PathBuf, level: Level) -> Result<Self, SinkError> {
        let loader = Self::open(&dir, Self::LOADER_FILE, level)?;
        Ok(Self { dir, level, loader, modules: Mutex::new(HashMap::new()) })
    }

    fn open(dir: &std::path::Path, file: &str, level: Level) -> Result<FileSink, SinkError> {
        let format: LineFormatter = Arc::new(Self::format);
        FileSink::with_rotation(
            dir.to_path_buf(),
            file.to_string(),
            level,
            format,
            Self::MAX_FILE_BYTES,
            RotationStrategy::KeepSome(Self::ARCHIVES_KEPT),
            DEFAULT_TIMEZONE_STRATEGY,
            DEFAULT_FILE_OPEN_STRATEGY,
        )
    }

    fn format(event: &LogEvent) -> String {
        let fields = if event.fields.is_empty() {
            String::new()
        } else {
            serde_json::to_string(&event.fields).map(|json| format!(" {json}")).unwrap_or_default()
        };
        format!("{} {:<5} {}{}", event.timestamp, event.level.as_str().to_ascii_uppercase(), event.message, fields)
    }

    fn emit_to_module(&self, module: &str, event: &LogEvent) -> bool {
        let Some((namespace, name)) = LoaderLog::registered(module) else {
            return false;
        };
        // Windows file names are case-insensitive, so `Echo` and `echo` share one file and one sink.
        let module = module.to_ascii_lowercase();
        let module = module.as_str();
        let Ok(mut modules) = self.modules.lock() else {
            return false;
        };
        if !modules.contains_key(module) {
            if modules.len() >= ModRegistry::MAX_MODS {
                return false;
            }
            let folder = self.dir.join(format!("@{namespace}"));
            if std::fs::create_dir_all(&folder).is_err() {
                return false;
            }
            let Ok(sink) = Self::open(&folder, &name, self.level) else {
                return false;
            };
            modules.insert(module.to_string(), sink);
        }
        match modules.get(module) {
            Some(sink) => {
                sink.emit(event);
                true
            }
            None => false,
        }
    }
}

impl Sink for LogRouter {
    fn level(&self) -> Level {
        self.level
    }

    fn emit(&self, event: &LogEvent) {
        let module = event.fields.get("module").and_then(|value| value.as_str());
        if let Some(module) = module
            && self.emit_to_module(module, event)
        {
            return;
        }
        self.loader.emit(event);
    }
}
