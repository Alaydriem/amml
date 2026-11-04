#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallStep {
    Checking,
    DownloadingCurrent,
    DownloadingWanted,
    Installing,
    CopyingExe,
    CopyingFiles,
    Removing,
    Restoring,
    Verifying,
    Done,
}

impl InstallStep {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Checking => "Checking the Store install",
            Self::DownloadingCurrent => "Downloading the installed version (kept for the restore)",
            Self::DownloadingWanted => "Downloading the wanted version",
            Self::Installing => "Installing the wanted version (keep this window open)",
            Self::CopyingExe => "Copying the game executable",
            Self::CopyingFiles => "Copying the game files",
            Self::Removing => "Removing the wanted version from the Store install",
            Self::Restoring => "Restoring the installed version",
            Self::Verifying => "Checking the Store install",
            Self::Done => "Done",
        }
    }
}
