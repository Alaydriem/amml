use serde::Serialize;
use ts_rs::TS;

// The named error states the app shows. The kit's nine, plus the three the app handles itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum FaultKindView {
    StoreMissing,
    GamingServicesMissing,
    PreviewMissing,
    DownloadFailed,
    DeployRefused,
    CopyFailed,
    DiskSpace,
    InstallInterrupted,
    RestoreFailed,
    OperationFailed,
    Busy,
    InstanceRunning,
    ConsentNeeded,
}
