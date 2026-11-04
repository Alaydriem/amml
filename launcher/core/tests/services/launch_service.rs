use crate::support::{FakeStarter, FakeWindowProbe, InstanceFolder};
use amml_launcher_core::{InstanceReader, InstanceState, LaunchService, LaunchState, LauncherError};
use std::time::Duration;

fn instance(root: &std::path::Path, name: &str, args: &str, env: &str) -> InstanceState {
    let dir = root.join(name);
    std::fs::create_dir_all(&dir).unwrap();
    InstanceFolder::create(&dir, "1.26.60.05").with_version_json(&format!(
        "{{\"gameVersion\":\"1.26.60.05\",\"launchArgs\":{},\"envVars\":{}}}",
        serde_json::Value::String(args.to_string()),
        serde_json::Value::String(env.to_string())
    ));
    InstanceReader::read(&dir).unwrap()
}

#[test]
fn a_window_is_reported_with_its_pid() {
    let root = tempfile::tempdir().unwrap();
    let state = instance(root.path(), "one", "", "");
    let probe = FakeWindowProbe::after(3);
    let mut service = LaunchService::new(FakeStarter::lasting(10), probe.clone(), Duration::from_secs(20));
    let launched = service.launch(&state).unwrap();
    let LaunchState::Window { pid, .. } = launched else { panic!("{launched:?}") };
    assert_eq!(probe.asked(), 3);
    let running = service.running();
    assert_eq!(running.len(), 1);
    assert_eq!((running[0].pid, running[0].instance.as_str()), (pid, "one"));
    assert!(service.stop(pid).unwrap());
}

#[test]
fn a_game_that_exits_early_is_exited_with_its_code() {
    let root = tempfile::tempdir().unwrap();
    let state = instance(root.path(), "one", "", "");
    let mut service = LaunchService::new(FakeStarter::exiting(3), FakeWindowProbe::never(), Duration::from_secs(20));
    match service.launch(&state).unwrap() {
        LaunchState::Exited { code, .. } => assert_eq!(code, Some(3)),
        other => panic!("{other:?}"),
    }
    assert!(service.running().is_empty());
}

#[test]
fn no_window_in_time_is_no_window_and_still_tracked() {
    let root = tempfile::tempdir().unwrap();
    let state = instance(root.path(), "one", "", "");
    let mut service =
        LaunchService::new(FakeStarter::lasting(10), FakeWindowProbe::never(), Duration::from_millis(600));
    let LaunchState::NoWindow { pid } = service.launch(&state).unwrap() else { panic!("expected NoWindow") };
    assert_eq!(service.running().len(), 1);
    assert!(service.stop(pid).unwrap());
    assert!(service.running().is_empty());
}

#[test]
fn stop_ends_only_its_own_children() {
    let root = tempfile::tempdir().unwrap();
    let state = instance(root.path(), "one", "", "");
    let mut stranger = FakeStarter::stranger();
    let mut service = LaunchService::new(FakeStarter::lasting(10), FakeWindowProbe::after(1), Duration::from_secs(20));
    let LaunchState::Window { pid, .. } = service.launch(&state).unwrap() else { panic!("expected a window") };

    assert!(!service.stop(stranger.id()).unwrap());
    assert!(stranger.try_wait().unwrap().is_none());
    assert!(service.stop(pid).unwrap());
    assert!(!service.stop(pid).unwrap());
    assert!(service.running().is_empty());
    stranger.kill().unwrap();
    stranger.wait().unwrap();
}

#[test]
fn the_working_directory_is_the_instance() {
    let root = tempfile::tempdir().unwrap();
    let state = instance(root.path(), "one", "-a \"b c\" d", "A=1\r\nB=x=y");
    let starter = FakeStarter::lasting(10);
    let mut service = LaunchService::new(starter.clone(), FakeWindowProbe::after(1), Duration::from_secs(20));
    let LaunchState::Window { pid, .. } = service.launch(&state).unwrap() else { panic!("expected a window") };
    let plans = starter.plans();
    assert_eq!(plans.len(), 1);
    assert_eq!(plans[0].cwd, state.folder);
    assert_eq!(plans[0].exe, state.folder.join("Minecraft.Windows.exe"));
    assert_eq!(plans[0].args, vec!["-a", "b c", "d"]);
    assert_eq!(plans[0].env, vec![("A".to_string(), "1".to_string()), ("B".to_string(), "x=y".to_string())]);
    service.stop(pid).unwrap();
}

#[test]
fn a_bad_launch_setting_starts_nothing() {
    let root = tempfile::tempdir().unwrap();
    let state = instance(root.path(), "one", "-a \"b", "");
    let starter = FakeStarter::lasting(10);
    let mut service = LaunchService::new(starter.clone(), FakeWindowProbe::after(1), Duration::from_secs(20));
    assert!(matches!(service.launch(&state), Err(LauncherError::BadLaunchSetting(_))));
    assert!(starter.plans().is_empty());
}

#[test]
fn the_33rd_running_game_is_refused_and_ended_ones_do_not_count() {
    let root = tempfile::tempdir().unwrap();
    let state = instance(root.path(), "one", "", "");
    let mut service = LaunchService::new(FakeStarter::lasting(8), FakeWindowProbe::after(1), Duration::from_secs(20));
    let mut pids = Vec::new();
    for _ in 0..LaunchService::<FakeStarter, FakeWindowProbe>::MAX_GAMES {
        let LaunchState::Window { pid, .. } = service.launch(&state).unwrap() else { panic!("expected a window") };
        pids.push(pid);
    }
    assert_eq!(service.running().len(), 32);
    assert!(
        matches!(service.launch(&state), Err(LauncherError::BadLaunchSetting(text)) if text == "too many running games")
    );

    assert!(service.stop(pids[0]).unwrap());
    assert!(service.launch(&state).is_ok());
    let alive: Vec<u32> = service.running().iter().map(|game| game.pid).collect();
    for pid in alive {
        service.stop(pid).unwrap();
    }
    assert!(service.running().is_empty());
}
