use amml_launcher_core::{
    InstanceReader, LaunchService, LaunchState, LauncherPaths, ProcessGameStarter, WindowsWindowProbe,
};
use std::time::Duration;

// Operator test. Set AMML_LAUNCH_ROOT to an AMML root that holds the instance `1.26.60.05-launcher-a`. The real
// starter and window probe start the game, wait for its window, and the service ends it again.
#[test]
#[ignore = "starts the game; needs a test instance"]
fn the_real_starter_and_probe_launch_and_stop_a_test_instance() {
    let root = std::path::PathBuf::from(std::env::var_os("AMML_LAUNCH_ROOT").expect("set AMML_LAUNCH_ROOT"));
    let paths = LauncherPaths::new(amml_gdk::AmmlPaths::new(root));
    let state = InstanceReader::read(&paths.instance("1.26.60.05-launcher-a")).unwrap();
    let mut service = LaunchService::new(
        ProcessGameStarter::new(),
        WindowsWindowProbe::new(),
        LaunchService::<ProcessGameStarter, WindowsWindowProbe>::DEFAULT_WINDOW_TIMEOUT + Duration::from_secs(60),
    );
    let LaunchState::Window { pid, .. } = service.launch(&state).unwrap() else { panic!("no window") };
    assert!(service.stop(pid).unwrap());
}
