use super::Locked;
use amml_gdk::ProcessInspector;
use amml_launcher_core::{
    GameStarter, InstanceGuard, InstanceReader, LaunchService, LaunchState, LauncherError, LauncherPaths, RunningGame,
    WindowProbe,
};
use std::sync::Mutex;
use std::time::Duration;

// The one way a game starts. It refuses an instance that already runs, whoever started it.
pub struct LaunchGate<S: GameStarter, W: WindowProbe, P: ProcessInspector> {
    service: Mutex<LaunchService<S, W>>,
    guard: InstanceGuard<P>,
    paths: LauncherPaths,
}

impl<S: GameStarter, W: WindowProbe, P: ProcessInspector> LaunchGate<S, W, P> {
    // how long a launch waits for the game window before it hands back; the game keeps running
    pub const WINDOW_WAIT: Duration = Duration::from_secs(2);

    pub fn new(starter: S, windows: W, guard: InstanceGuard<P>, paths: LauncherPaths) -> LaunchGate<S, W, P> {
        LaunchGate { service: Mutex::new(LaunchService::new(starter, windows, Self::WINDOW_WAIT)), guard, paths }
    }

    pub fn launch(&self, name: &str) -> Result<LaunchState, LauncherError> {
        let folder = self.paths.checked_instance(name)?;
        let state = InstanceReader::read(&folder)?;
        let mut service = Locked::of(&self.service);
        if service.running().iter().any(|game| game.instance == name) {
            return Err(LauncherError::InstanceRunning(name.to_string()));
        }
        self.guard.ensure_stopped(&folder, name)?;
        service.launch(&state)
    }

    // ends a game this gate started
    pub fn stop(&self, pid: u32) -> Result<bool, LauncherError> {
        Locked::of(&self.service).stop(pid)
    }

    pub fn running(&self) -> Vec<RunningGame> {
        Locked::of(&self.service).running()
    }
}
