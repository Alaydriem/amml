use curia::{ConsoleSink, FileSink, Level, LogEvent, Sink};

// The two places a log line goes: the console of a dev run, and the log file.
pub enum LauncherSink {
    Console(ConsoleSink),
    File(FileSink),
}

impl Sink for LauncherSink {
    fn level(&self) -> Level {
        match self {
            Self::Console(sink) => sink.level(),
            Self::File(sink) => sink.level(),
        }
    }

    fn emit(&self, event: &LogEvent) {
        match self {
            Self::Console(sink) => sink.emit(event),
            Self::File(sink) => sink.emit(event),
        }
    }
}
