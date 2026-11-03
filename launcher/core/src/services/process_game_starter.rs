use super::GameStarter;
use crate::{LaunchPlan, LauncherError};
use std::os::windows::process::CommandExt;
use std::process::{Child, Command, Stdio};

// Starts the game with no console window and no standard streams.
#[derive(Default)]
pub struct ProcessGameStarter;

impl ProcessGameStarter {
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    pub fn new() -> ProcessGameStarter {
        ProcessGameStarter
    }
}

impl GameStarter for ProcessGameStarter {
    fn start(&self, plan: &LaunchPlan) -> Result<Child, LauncherError> {
        let child = Command::new(&plan.exe)
            .current_dir(&plan.cwd)
            .args(&plan.args)
            .envs(plan.env.iter().map(|(key, value)| (key, value)))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(Self::CREATE_NO_WINDOW)
            .spawn()?;
        Ok(child)
    }
}
