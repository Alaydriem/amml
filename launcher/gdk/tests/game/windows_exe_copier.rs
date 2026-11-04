use amml_gdk::{PackageInventory, PeCheck, RegisteredPackage, WindowsExeCopier, WindowsPackageInventory};
use std::time::Duration;

fn current() -> RegisteredPackage {
    WindowsPackageInventory::new().registered("Microsoft.MinecraftUWP_8wekyb3d8bbwe").unwrap().unwrap()
}

// Read-only for the Store install: copies the exe of the registered version into a temp folder.
#[test]
#[ignore = "needs a Store install of Minecraft Bedrock"]
fn copies_current_store_exe() {
    let package = current();
    let dest = tempfile::tempdir().unwrap();

    let copy = WindowsExeCopier::new(Duration::from_secs(120)).copy_plain(&package, dest.path()).unwrap();

    let source = std::fs::metadata(package.install_location.join("Minecraft.Windows.exe")).unwrap().len();
    assert_eq!(copy.bytes, source);
    assert!(PeCheck::is_x64(&copy.path).unwrap());
}

// Starts robocopy with the game's package identity on the registered version. The registration is not changed.
#[test]
#[ignore = "needs a Store install of Minecraft Bedrock"]
fn copies_current_store_exe_in_package_context() {
    let package = current();
    let dest = tempfile::tempdir().unwrap();

    let copy =
        WindowsExeCopier::new(Duration::from_secs(120)).copy_in_package_context(&package, "Game", dest.path()).unwrap();

    let source = std::fs::metadata(package.install_location.join("Minecraft.Windows.exe")).unwrap().len();
    assert_eq!(copy.bytes, source);
    assert_eq!(copy.method, "package-context");
    assert!(PeCheck::is_x64(&copy.path).unwrap());
    assert!(!dest.path().join("robocopy.log").exists());
}

// The destination holds a space and both kinds of apostrophe, as a user name may.
#[test]
#[ignore = "needs a Store install of Minecraft Bedrock"]
fn package_context_copy_into_a_folder_with_apostrophes() {
    let package = current();
    let parent = tempfile::tempdir().unwrap();
    let dest = parent.path().join("O\u{2019}Brien it's \u{2018}x\u{2019}");

    let copy =
        WindowsExeCopier::new(Duration::from_secs(120)).copy_in_package_context(&package, "Game", &dest).unwrap();

    let source = std::fs::metadata(package.install_location.join("Minecraft.Windows.exe")).unwrap().len();
    assert_eq!(copy.bytes, source);
    assert!(PeCheck::is_x64(&copy.path).unwrap());
}
