use crate::support::{AlwaysWindow, NoProcesses, PingStarter, Samples};
use amml_launcher_bridge::LaunchGate;
use amml_launcher_core::{InstanceGuard, LaunchState, LauncherError};
use std::sync::Arc;

fn make_instance(root: &std::path::Path, name: &str) {
    let folder = root.join("instances").join(name);
    std::fs::create_dir_all(&folder).unwrap();
    let text = format!(
        "{{\"name\":\"{name}\",\"gameVersion\":\"1.26.60.05\",\"type\":\"release\",\"enableIsolation\":false,\"launchArgs\":\"\",\"envVars\":\"\"}}"
    );
    std::fs::write(folder.join("version.json"), text).unwrap();
    std::fs::write(folder.join("Minecraft.Windows.exe.bak"), b"x").unwrap();
}

#[test]
fn launching_an_instance_the_launcher_already_runs_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    make_instance(dir.path(), "play");
    make_instance(dir.path(), "other");
    let gate = LaunchGate::new(
        PingStarter,
        AlwaysWindow,
        InstanceGuard::new(Arc::new(NoProcesses)),
        Samples::paths(dir.path()),
    );

    let first = gate.launch("play").unwrap();
    let again = gate.launch("play");
    let other = gate.launch("other").unwrap();

    assert!(matches!(first, LaunchState::Window { .. }));
    assert!(matches!(again, Err(LauncherError::InstanceRunning(name)) if name == "play"));
    assert!(matches!(other, LaunchState::Window { .. }));
    let mut running: Vec<String> = gate.running().into_iter().map(|game| game.instance).collect();
    running.sort();
    assert_eq!(running, vec!["other", "play"]);
    // the two ping processes are ended by their own PIDs
    for game in gate.running() {
        assert!(gate.stop(game.pid).unwrap());
    }
    assert!(gate.running().is_empty());
}

#[test]
fn a_name_that_is_not_an_instance_is_refused_before_anything_starts() {
    let dir = tempfile::tempdir().unwrap();
    let gate = LaunchGate::new(
        PingStarter,
        AlwaysWindow,
        InstanceGuard::new(Arc::new(NoProcesses)),
        Samples::paths(dir.path()),
    );

    assert!(gate.launch("..").is_err());
    assert!(gate.launch("missing").is_err());
    assert!(gate.running().is_empty());
}
