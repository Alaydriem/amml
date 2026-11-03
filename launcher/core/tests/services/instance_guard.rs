use crate::support::FakeProcesses;
use amml_launcher_core::{InstanceGuard, LauncherError};
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn a_game_from_another_folder_does_not_block() {
    let processes = FakeProcesses::new();
    processes.start(10, PathBuf::from("C:\\games\\other\\Minecraft.Windows.exe"));
    processes.start(11, PathBuf::new());
    let guard = InstanceGuard::new(Arc::new(processes));
    assert!(guard.ensure_stopped(&PathBuf::from("C:\\amml\\instances\\one"), "one").is_ok());
}

#[test]
fn the_same_exe_path_in_other_case_blocks() {
    let processes = FakeProcesses::new();
    processes.start(10, PathBuf::from("c:/AMML/Instances/ONE/minecraft.windows.exe"));
    let guard = InstanceGuard::new(Arc::new(processes));
    match guard.ensure_stopped(&PathBuf::from("C:\\amml\\instances\\one"), "one") {
        Err(LauncherError::InstanceRunning(name)) => assert_eq!(name, "one"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_folder_that_only_starts_with_the_same_name_does_not_block() {
    let processes = FakeProcesses::new();
    processes.start(10, PathBuf::from("C:\\amml\\instances\\one-b\\Minecraft.Windows.exe"));
    let guard = InstanceGuard::new(Arc::new(processes));
    assert!(guard.ensure_stopped(&PathBuf::from("C:\\amml\\instances\\one"), "one").is_ok());
}

#[test]
fn clones_share_one_inspector() {
    let processes = FakeProcesses::new();
    let guard = InstanceGuard::new(Arc::new(processes.clone()));
    let copy = guard.clone();
    processes.start(10, PathBuf::from("C:\\amml\\instances\\one\\Minecraft.Windows.exe"));
    assert!(copy.ensure_stopped(&PathBuf::from("C:\\amml\\instances\\one"), "one").is_err());
}
