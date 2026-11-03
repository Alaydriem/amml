use super::{GameStarter, LaunchSettings, WindowProbe};
use crate::{InstanceReader, InstanceState, LaunchPlan, LaunchState, LauncherError, RunningGame, VersionJsonFile};
use std::process::Child;
use std::time::{Duration, Instant, SystemTime};

// Starts instances and keeps the handles of the games it started, at most `MAX_GAMES` of them. It stops only those.
pub struct LaunchService<S: GameStarter, W: WindowProbe> {
    starter: S,
    windows: W,
    window_timeout: Duration,
    children: Vec<(Child, RunningGame)>,
}

impl<S: GameStarter, W: WindowProbe> LaunchService<S, W> {
    pub const DEFAULT_WINDOW_TIMEOUT: Duration = Duration::from_secs(120);
    pub const MAX_GAMES: usize = 32;
    const POLL: Duration = Duration::from_millis(250);

    pub fn new(starter: S, windows: W, window_timeout: Duration) -> LaunchService<S, W> {
        LaunchService { starter, windows, window_timeout, children: Vec::new() }
    }

    // Starts the instance and waits for its window, for the process to end, or for the timeout.
    pub fn launch(&mut self, instance: &InstanceState) -> Result<LaunchState, LauncherError> {
        let version_json = VersionJsonFile::read(&instance.folder.join(InstanceReader::VERSION_JSON))?;
        let plan = LaunchPlan {
            exe: instance.folder.join(InstanceReader::EXE_NAME),
            cwd: instance.folder.clone(),
            args: LaunchSettings::args(&version_json.launch_args())?,
            env: LaunchSettings::env(&version_json.env_vars())?,
        };
        self.drop_exited();
        if self.children.len() >= Self::MAX_GAMES {
            return Err(LauncherError::BadLaunchSetting("too many running games".to_string()));
        }
        let child = self.starter.start(&plan)?;
        let pid = child.id();
        let game = RunningGame { pid, instance: instance.name.clone(), started: SystemTime::now() };
        self.children.push((child, game));

        let started = Instant::now();
        loop {
            let seconds = started.elapsed().as_secs();
            if let Some(code) = self.exit_of(pid) {
                self.children.retain(|(_, game)| game.pid != pid);
                return Ok(LaunchState::Exited { code, seconds });
            }
            if self.windows.has_window(pid) {
                return Ok(LaunchState::Window { pid, seconds });
            }
            if started.elapsed() >= self.window_timeout {
                return Ok(LaunchState::NoWindow { pid });
            }
            std::thread::sleep(Self::POLL);
        }
    }

    // the games that are still running; ended ones are dropped
    pub fn running(&mut self) -> Vec<RunningGame> {
        self.drop_exited();
        self.children.iter().map(|(_, game)| game.clone()).collect()
    }

    // Ends a game this service started. `false` for a PID it did not start.
    pub fn stop(&mut self, pid: u32) -> Result<bool, LauncherError> {
        let Some(position) = self.children.iter().position(|(_, game)| game.pid == pid) else {
            return Ok(false);
        };
        let (mut child, _) = self.children.remove(position);
        if child.try_wait()?.is_none() {
            child.kill()?;
        }
        child.wait()?;
        Ok(true)
    }

    // `Some(code)` once the process has ended
    fn exit_of(&mut self, pid: u32) -> Option<Option<i32>> {
        let (child, _) = self.children.iter_mut().find(|(_, game)| game.pid == pid)?;
        child.try_wait().ok().flatten().map(|status| status.code())
    }

    fn drop_exited(&mut self) {
        self.children.retain_mut(|(child, _)| matches!(child.try_wait(), Ok(None)));
    }
}
