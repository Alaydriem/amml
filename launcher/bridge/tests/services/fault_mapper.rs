use amml_gdk::{GdkError, StoreState};
use amml_launcher_bridge::{BridgeError, FaultKindView, FaultMapper, FaultView};
use amml_launcher_core::LauncherError;
use std::path::PathBuf;

fn not_ready(state: StoreState) -> GdkError {
    GdkError::NotReady(Box::new(state))
}

#[test]
fn each_pinned_error_maps_to_its_fault_kind() {
    assert_eq!(FaultMapper::from_gdk(&not_ready(StoreState::BedrockMissing)).kind, FaultKindView::StoreMissing);
    assert_eq!(
        FaultMapper::from_gdk(&not_ready(StoreState::GamingServicesMissing)).kind,
        FaultKindView::GamingServicesMissing
    );
    assert_eq!(FaultMapper::from_gdk(&not_ready(StoreState::PreviewMissing)).kind, FaultKindView::PreviewMissing);

    let space = FaultMapper::from_gdk(&not_ready(StoreState::NotEnoughSpace {
        drive: PathBuf::from("C:\\"),
        needed: 6,
        free: 3,
    }));
    assert_eq!(space.kind, FaultKindView::DiskSpace);
    assert_eq!(space.values.get("drive").map(String::as_str), Some("C:\\"));
    assert_eq!(space.values.get("needed").map(String::as_str), Some("6"));
    assert_eq!(space.values.get("free").map(String::as_str), Some("3"));

    assert_eq!(FaultMapper::from_gdk(&GdkError::Download("x".into())).kind, FaultKindView::DownloadFailed);

    let deploy = FaultMapper::from_gdk(&GdkError::Deploy {
        action: "register",
        hresult: 0x80073CF3_u32 as i32,
        text: "package failed updates".into(),
    });
    assert_eq!(deploy.kind, FaultKindView::DeployRefused);
    assert_eq!(deploy.code.as_deref(), Some("0x80073CF3"));

    let exe = GdkError::ExeCopyFailed { method: "plain", detail: "denied".into() };
    assert_eq!(FaultMapper::from_gdk(&exe).kind, FaultKindView::ConsentNeeded);

    let copy = FaultMapper::from_gdk(&GdkError::IncompleteCopy { expected: 12178, copied: 9412 });
    assert_eq!(copy.kind, FaultKindView::CopyFailed);
    assert_eq!(copy.values.get("copied").map(String::as_str), Some("9412"));
    assert_eq!(copy.values.get("expected").map(String::as_str), Some("12178"));

    let running = FaultMapper::from_launcher(&LauncherError::InstanceRunning("play".into()));
    assert_eq!(running.kind, FaultKindView::InstanceRunning);
    assert_eq!(running.values.get("name").map(String::as_str), Some("play"));

    assert_eq!(FaultMapper::from_bridge(&BridgeError::Busy).kind, FaultKindView::Busy);
}

#[test]
fn a_gdk_error_inside_a_launcher_error_maps_the_same() {
    let wrapped = LauncherError::Gdk(GdkError::IncompleteCopy { expected: 2, copied: 1 });
    assert_eq!(FaultMapper::from_launcher(&wrapped).kind, FaultKindView::CopyFailed);
    assert_eq!(FaultMapper::from_bridge(&BridgeError::Launcher(wrapped)).kind, FaultKindView::CopyFailed);
}

#[test]
fn an_unlisted_error_is_operation_failed_with_its_text_as_code() {
    let fault = FaultMapper::from_launcher(&LauncherError::UnknownMod("@x/y".into()));
    assert_eq!(fault.kind, FaultKindView::OperationFailed);
    assert_eq!(fault.code.as_deref(), Some("unknown mod @x/y"));
    assert!(fault.values.is_empty());
    assert!(!fault.restored);

    let not_healthy = FaultMapper::from_gdk(&not_ready(StoreState::NotHealthy { full_name: "p".into() }));
    assert_eq!(not_healthy.kind, FaultKindView::OperationFailed);
    assert!(not_healthy.code.is_some());
}

#[test]
fn a_cancelled_download_is_recognised_through_a_launcher_error() {
    assert!(FaultMapper::is_cancelled(&LauncherError::Gdk(GdkError::Cancelled)));
    assert!(!FaultMapper::is_cancelled(&LauncherError::UnknownMod("x".into())));
    let converted: FaultView = BridgeError::Busy.into();
    assert_eq!(converted.kind, FaultKindView::Busy);
}

#[test]
fn a_store_install_that_was_not_put_back_is_its_own_fault_with_the_package_path() {
    let failed = GdkError::RestoreFailed {
        original: "Microsoft.MinecraftUWP_1".to_string(),
        package: PathBuf::from("C:\\amml\\downloads\\original.msixvc"),
        cause: "denied".to_string(),
    };

    let direct = FaultMapper::from_gdk(&failed);
    let inside = FaultMapper::from_launcher(&LauncherError::Gdk(failed));

    assert_eq!(direct.kind, FaultKindView::RestoreFailed);
    assert_eq!(direct.code.as_deref(), Some("C:\\amml\\downloads\\original.msixvc"));
    assert!(!direct.restored);
    assert_eq!(inside.kind, FaultKindView::RestoreFailed);
}
