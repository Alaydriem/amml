use amml_launcher_app_lib::app::LauncherUpdater;
use amml_launcher_bridge::BundleKind;
use tauri::utils::config::BundleType;

#[test]
fn only_nsis_and_msi_bundles_map() {
    assert_eq!(LauncherUpdater::bundle_kind(Some(BundleType::Nsis)), Some(BundleKind::Nsis));
    assert_eq!(LauncherUpdater::bundle_kind(Some(BundleType::Msi)), Some(BundleKind::Msi));
    assert_eq!(LauncherUpdater::bundle_kind(Some(BundleType::Deb)), None);
    assert_eq!(LauncherUpdater::bundle_kind(None), None);
}
