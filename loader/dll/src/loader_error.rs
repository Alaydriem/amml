/// Why the loader state could not start.
#[derive(Debug, thiserror::Error)]
pub enum LoaderError {
    #[error("the loader is already initialized in this process")]
    AlreadyInitialized,
    #[error("cannot open the log files: {0}")]
    Log(#[from] curia::SinkError),
}
