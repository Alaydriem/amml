use amml_gdk::{PackageDeployer, PackageInventory, WindowsPackageDeployer, WindowsPackageInventory};
use std::path::PathBuf;

const FAMILY: &str = "Microsoft.MinecraftUWP_8wekyb3d8bbwe";

// Operator-run. Registers the current version's own package over itself. Set AMML_GDK_CURRENT_MSIXVC to the cached
// package of the version that is installed now. Minecraft must not be running.
#[test]
#[ignore = "changes the Store install; run by the operator with AMML_GDK_CURRENT_MSIXVC set"]
fn re_add_current_package_keeps_registration() {
    let package = PathBuf::from(std::env::var("AMML_GDK_CURRENT_MSIXVC").unwrap());
    let inventory = WindowsPackageInventory::new();
    let before = inventory.registered(FAMILY).unwrap().unwrap();

    WindowsPackageDeployer::new().add(&package).unwrap();

    let after = inventory.registered(FAMILY).unwrap().unwrap();
    assert_eq!(after.full_name, before.full_name);
    assert!(after.status_ok);
}
