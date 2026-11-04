use amml_gdk::{GdkError, PackageDeployer, WindowsPackageDeployer};

#[test]
fn remove_refuses_protected_full_name() {
    let deployer = WindowsPackageDeployer::new();
    let protected = vec!["Y".to_string(), "X".to_string()];
    assert!(matches!(deployer.remove("X", &protected), Err(GdkError::Protected(name)) if name == "X"));
}

#[test]
fn protected_name_is_compared_without_case() {
    let deployer = WindowsPackageDeployer::new();
    let protected = vec!["Microsoft.MinecraftUWP_1.26.6005.0_x64__8wekyb3d8bbwe".to_string()];
    assert!(matches!(
        deployer.remove("microsoft.minecraftuwp_1.26.6005.0_x64__8wekyb3d8bbwe", &protected),
        Err(GdkError::Protected(_))
    ));
}
