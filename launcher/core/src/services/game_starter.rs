use crate::{LaunchPlan, LauncherError};
use std::process::Child;

// Starts the game process.
pub trait GameStarter {
    fn start(&self, plan: &LaunchPlan) -> Result<Child, LauncherError>;
}
