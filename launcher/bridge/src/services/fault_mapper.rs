use crate::{BridgeError, FaultKindView, FaultView};
use amml_gdk::{GdkError, StoreState};
use amml_launcher_core::LauncherError;
use std::collections::BTreeMap;

// Maps an error to the named fault the UI words. The English text of an error goes only into `code`.
pub struct FaultMapper;

impl FaultMapper {
    pub fn from_launcher(error: &LauncherError) -> FaultView {
        match error {
            LauncherError::Gdk(inner) => Self::from_gdk(inner),
            LauncherError::InstanceRunning(name) => {
                Self::fault(FaultKindView::InstanceRunning, None, &[("name", name.clone())])
            }
            other => Self::other(other.to_string()),
        }
    }

    pub fn from_gdk(error: &GdkError) -> FaultView {
        match error {
            GdkError::NotReady(state) => Self::from_store_state(state, error),
            GdkError::Download(_) => Self::fault(FaultKindView::DownloadFailed, None, &[]),
            GdkError::Deploy { action, hresult, .. } => Self::fault(
                FaultKindView::DeployRefused,
                Some(format!("0x{hresult:08X}")),
                &[("action", (*action).to_string())],
            ),
            GdkError::RestoreFailed { package, .. } => {
                Self::fault(FaultKindView::RestoreFailed, Some(package.display().to_string()), &[])
            }
            GdkError::ExeCopyFailed { .. } => Self::fault(FaultKindView::ConsentNeeded, None, &[]),
            GdkError::IncompleteCopy { expected, copied } => Self::fault(
                FaultKindView::CopyFailed,
                None,
                &[("copied", copied.to_string()), ("expected", expected.to_string())],
            ),
            other => Self::other(other.to_string()),
        }
    }

    pub fn from_bridge(error: &BridgeError) -> FaultView {
        match error {
            BridgeError::Busy => Self::fault(FaultKindView::Busy, None, &[]),
            BridgeError::Launcher(inner) => Self::from_launcher(inner),
            BridgeError::Gdk(inner) => Self::from_gdk(inner),
            other => Self::other(other.to_string()),
        }
    }

    // a cancelled download ends a job as cancelled, not as a fault
    pub fn is_cancelled(error: &LauncherError) -> bool {
        matches!(error, LauncherError::Gdk(GdkError::Cancelled))
    }

    fn from_store_state(state: &StoreState, error: &GdkError) -> FaultView {
        match state {
            StoreState::BedrockMissing => Self::fault(FaultKindView::StoreMissing, None, &[]),
            StoreState::GamingServicesMissing => Self::fault(FaultKindView::GamingServicesMissing, None, &[]),
            StoreState::PreviewMissing => Self::fault(FaultKindView::PreviewMissing, None, &[]),
            StoreState::NotEnoughSpace { drive, needed, free } => Self::fault(
                FaultKindView::DiskSpace,
                None,
                &[("drive", drive.display().to_string()), ("needed", needed.to_string()), ("free", free.to_string())],
            ),
            _ => Self::other(error.to_string()),
        }
    }

    fn other(text: String) -> FaultView {
        Self::fault(FaultKindView::OperationFailed, Some(text), &[])
    }

    fn fault(kind: FaultKindView, code: Option<String>, values: &[(&str, String)]) -> FaultView {
        let values: BTreeMap<String, String> =
            values.iter().map(|(key, value)| ((*key).to_string(), value.clone())).collect();
        FaultView::new(kind, code, values)
    }
}

impl From<&BridgeError> for FaultView {
    fn from(error: &BridgeError) -> FaultView {
        FaultMapper::from_bridge(error)
    }
}

impl From<BridgeError> for FaultView {
    fn from(error: BridgeError) -> FaultView {
        FaultMapper::from_bridge(&error)
    }
}
