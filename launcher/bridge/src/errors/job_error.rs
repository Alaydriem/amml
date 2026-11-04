use amml_gdk::GdkError;
use amml_launcher_core::LauncherError;

// What ends a job early.
#[derive(Debug, thiserror::Error)]
pub enum JobError {
    #[error(transparent)]
    Launcher(#[from] LauncherError),
    #[error(transparent)]
    Gdk(#[from] GdkError),
}

impl JobError {
    // a cancelled download ends a job as cancelled, not as failed
    pub fn is_cancelled(&self) -> bool {
        matches!(self, Self::Gdk(GdkError::Cancelled) | Self::Launcher(LauncherError::Gdk(GdkError::Cancelled)))
    }
}
