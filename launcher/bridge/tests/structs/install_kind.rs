use amml_launcher_bridge::{BundleKind, InstallKind};

#[test]
fn msix_never_self_updates_even_with_a_bundle_type() {
    let kind = InstallKind::detect(true, Some(BundleKind::Nsis));
    assert_eq!(kind, InstallKind::Msix);
    assert!(!kind.self_updates());
}

#[test]
fn standalone_exe_does_not_self_update() {
    let kind = InstallKind::detect(false, None);
    assert_eq!(kind, InstallKind::Standalone);
    assert!(!kind.self_updates());
}

#[test]
fn only_a_store_install_skips_the_github_check() {
    assert!(!InstallKind::Msix.checks_github());
    assert!(InstallKind::Standalone.checks_github());
    assert!(InstallKind::Nsis.checks_github());
    assert!(InstallKind::Msi.checks_github());
}

#[test]
fn nsis_and_msi_self_update() {
    let nsis = InstallKind::detect(false, Some(BundleKind::Nsis));
    let msi = InstallKind::detect(false, Some(BundleKind::Msi));
    assert_eq!(nsis, InstallKind::Nsis);
    assert_eq!(msi, InstallKind::Msi);
    assert!(nsis.self_updates());
    assert!(msi.self_updates());
}
