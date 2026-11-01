use std::sync::{Mutex, PoisonError};

use log::{Level, LevelFilter, Log, Metadata, Record};

/// The process logger for tests that check egui warns of nothing: it keeps each warning and error line.
/// cargo-nextest runs each test in its own process, so one test's lines never reach another's.
pub struct WarnLog {
    lines: Mutex<Vec<String>>,
}

static LOG: WarnLog = WarnLog { lines: Mutex::new(Vec::new()) };

impl WarnLog {
    /// Installs the logger; a second call keeps the first one.
    pub fn install() -> &'static WarnLog {
        if log::set_logger(&LOG).is_ok() {
            log::set_max_level(LevelFilter::Warn);
        }
        &LOG
    }

    pub fn lines(&self) -> Vec<String> {
        self.lines.lock().unwrap_or_else(PoisonError::into_inner).clone()
    }
}

impl Log for WarnLog {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.level() <= Level::Warn
    }

    fn log(&self, record: &Record<'_>) {
        if self.enabled(record.metadata()) {
            self.lines.lock().unwrap_or_else(PoisonError::into_inner).push(record.args().to_string());
        }
    }

    fn flush(&self) {}
}
