use amml_launcher_core::{Patcher, ProcessPatcher};
use std::path::PathBuf;

// Operator test. Set AMML_PATCH_INSTANCE to a test instance that `mise run install` patched, and build
// `amml-patch` in release mode first. The launcher's patcher run over the instance's `.bak` must give the same bytes.
#[test]
#[ignore = "needs a test instance and a release build of amml-patch"]
fn the_patcher_output_equals_the_install_game_output() {
    let instance = PathBuf::from(std::env::var_os("AMML_PATCH_INSTANCE").expect("set AMML_PATCH_INSTANCE"));
    let patcher_exe = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("target")
        .join("release")
        .join("amml-patch.exe");
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("Minecraft.Windows.exe");
    ProcessPatcher::new(ProcessPatcher::DEFAULT_TIMEOUT)
        .patch(&patcher_exe, &instance.join("Minecraft.Windows.exe.bak"), &output)
        .unwrap();
    assert_eq!(std::fs::read(&output).unwrap(), std::fs::read(instance.join("Minecraft.Windows.exe")).unwrap());
}
