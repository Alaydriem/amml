use super::LauncherSink;
use curia::{
    ConsoleSink, Dispatcher, FileOpenStrategy, FileSink, Level, LineFormatter, LogEvent, Logger, RotationStrategy,
    TimezoneStrategy,
};
use std::path::PathBuf;
use std::sync::Arc;

// Installs the Curia sinks: the console and a rotating file at `<amml>\launcher\logs\launcher.log`.
pub struct LogSetup;

impl LogSetup {
    // curia's own default is 40,000 bytes, which throws a debug session away several times a minute
    pub const MAX_FILE_SIZE: u64 = 25 * 1024 * 1024;
    // archives kept beside the active file: 100 MB at most, and the run before last
    pub const ARCHIVES_KEPT: usize = 3;
    const FILE_NAME: &'static str = "launcher";

    // An unwritable log folder leaves the console sink alone; the app still starts.
    pub fn install(dir: PathBuf) -> Result<(), curia::InstallError> {
        let mut file_error: Option<String> = None;
        let mut sinks = vec![LauncherSink::Console(ConsoleSink::new(Level::Info, Self::formatter()))];
        match FileSink::with_rotation(
            dir,
            Self::FILE_NAME.to_string(),
            Level::Info,
            Self::formatter(),
            Self::MAX_FILE_SIZE,
            RotationStrategy::KeepSome(Self::ARCHIVES_KEPT),
            TimezoneStrategy::UseUtc,
            FileOpenStrategy::Append,
        ) {
            Ok(file) => sinks.push(LauncherSink::File(file)),
            Err(error) => file_error = Some(error.to_string()),
        }
        Logger::install(Box::new(Dispatcher::new(sinks)))?;
        if let Some(error) = file_error {
            curia::warn!("the log file is not available, so the console is used", { error: error });
        }
        Ok(())
    }

    // `<time> <level> <target> <message> {fields}`
    fn formatter() -> LineFormatter {
        Arc::new(|event: &LogEvent| {
            let fields = serde_json::to_string(&event.fields).unwrap_or_default();
            format!(
                "{} {:5} {} {} {}",
                event.timestamp.to_rfc3339(),
                event.level.as_str(),
                event.target,
                event.message,
                fields
            )
        })
    }
}
