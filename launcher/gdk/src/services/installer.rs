use crate::GdkError;
use crate::structs::{DownloadProgress, InstallOutcome, InstallRequest, InstallStep};

// What the command line needs from an install service.
pub trait Installer {
    fn install(
        &self,
        request: &InstallRequest,
        on_step: &mut dyn FnMut(InstallStep, Option<DownloadProgress>),
    ) -> Result<InstallOutcome, GdkError>;

    fn recover(&self) -> Result<Option<String>, GdkError>;
}
