use amml_gdk::{DownloadProgress, InstallStep};

// What the core is doing, for a progress line or a progress screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreProgress {
    InstallingVersion { step: InstallStep, download: Option<DownloadProgress> },
    CreatingInstance,
    Downloading { file: String, progress: DownloadProgress },
    Verifying { file: String },
    InstallingAmml { version: String },
    Patching,
    InstallingMod { id: String },
    Finishing,
    Done,
}
