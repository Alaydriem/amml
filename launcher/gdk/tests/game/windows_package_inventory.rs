use amml_gdk::{PackageInventory, WindowsPackageInventory};

// Read-only: lists the registered Bedrock package of the current user. Needs a Store install.
#[test]
#[ignore = "needs a Store install of Minecraft Bedrock"]
fn reads_the_registered_bedrock_package() {
    let package = WindowsPackageInventory::new().registered("Microsoft.MinecraftUWP_8wekyb3d8bbwe").unwrap().unwrap();
    println!("{package:?}");
    assert!(package.full_name.starts_with("Microsoft.MinecraftUWP_"));
    assert!(package.install_location.join("Minecraft.Windows.exe").exists());
}
