use amml_gdk::{GdkError, PackageInventory, ProcessInspector, RegisteredPackage};
use amml_launcher_core::{GameStarter, LaunchPlan, LauncherError, WindowProbe};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

// Package families that are registered, with the package for each.
pub struct FakeInventory {
    packages: Vec<(String, RegisteredPackage)>,
}

impl FakeInventory {
    pub fn new(packages: Vec<(&str, RegisteredPackage)>) -> FakeInventory {
        FakeInventory {
            packages: packages.into_iter().map(|(family, package)| (family.to_string(), package)).collect(),
        }
    }
}

impl PackageInventory for FakeInventory {
    fn registered(&self, family: &str) -> Result<Option<RegisteredPackage>, GdkError> {
        Ok(self.packages.iter().find(|(held, _)| held == family).map(|(_, package)| package.clone()))
    }
}

// No game process runs anywhere.
pub struct NoProcesses;

impl ProcessInspector for NoProcesses {
    fn image_paths(&self, _exe_name: &str) -> Result<Vec<(u32, PathBuf)>, GdkError> {
        Ok(Vec::new())
    }
}

// Starts a harmless process that lives for a few seconds instead of the game.
pub struct PingStarter;

impl GameStarter for PingStarter {
    fn start(&self, _plan: &LaunchPlan) -> Result<Child, LauncherError> {
        Ok(Command::new("ping.exe")
            .args(["-n", "4", "127.0.0.1"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?)
    }
}

// Every process shows a window at once.
pub struct AlwaysWindow;

impl WindowProbe for AlwaysWindow {
    fn has_window(&self, _pid: u32) -> bool {
        true
    }
}
