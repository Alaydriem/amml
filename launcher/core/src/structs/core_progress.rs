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
    // the job waits for the manager (true) or got it (false)
    WaitingForManager { waiting: bool },
    // the copy of a data folder is measured; `merge` is true when the target already holds files
    MeasuringData { merge: bool },
    // `detail` names the source and the target; `progress` counts the bytes of the files copied so far
    CopyingData { detail: String, progress: DownloadProgress },
    SavingSettings,
}
