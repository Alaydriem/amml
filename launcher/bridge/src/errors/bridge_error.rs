use crate::NameProblemView;
use amml_gdk::GdkError;
use amml_launcher_core::LauncherError;

#[derive(Debug, thiserror::Error)]
pub enum BridgeError {
    // a job holds the one manager, or a job is already running
    #[error("another task is running")]
    Busy,
    #[error(transparent)]
    Launcher(#[from] LauncherError),
    #[error(transparent)]
    Gdk(#[from] GdkError),
    #[error("file error: {0}")]
    Io(#[from] std::io::Error),
    // a path or an address the app does not act on
    #[error("{0} is not allowed")]
    Refused(String),
    #[error("{0} is not available")]
    Unavailable(String),
    #[error("the instance name cannot be used: {0:?}")]
    NameProblem(NameProblemView),
    #[error("the game ended at once (exit code {0:?})")]
    GameExited(Option<i32>),
}
