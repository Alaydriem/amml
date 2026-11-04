use super::DeveloperModeControl;
use crate::GdkError;
use std::os::windows::process::CommandExt;
use std::process::Command;

pub struct DeveloperMode;

impl DeveloperMode {
    const KEY: &'static str = r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\AppModelUnlock";
    const VALUE: &'static str = "AllowDevelopmentWithoutDevLicense";
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    pub fn new() -> DeveloperMode {
        DeveloperMode
    }

    // a missing key or value is off
    pub fn is_on(&self) -> Result<bool, GdkError> {
        let output = Command::new("reg.exe")
            .args(["query", Self::KEY, "/v", Self::VALUE])
            .creation_flags(Self::CREATE_NO_WINDOW)
            .output()?;
        Ok(Self::is_on_in(&String::from_utf8_lossy(&output.stdout)))
    }

    // reads the text `reg.exe query` prints for the value
    pub fn is_on_in(query_output: &str) -> bool {
        query_output
            .lines()
            .filter(|line| line.contains(Self::VALUE))
            .any(|line| line.split_whitespace().last().is_some_and(|data| data.eq_ignore_ascii_case("0x1")))
    }

    // Turns Developer Mode on through a UAC prompt. Call it only after the player said yes.
    pub fn enable_with_consent(&self) -> Result<bool, GdkError> {
        let arguments = format!("add {} /v {} /t REG_DWORD /d 1 /f", Self::KEY, Self::VALUE);
        let command = format!("Start-Process reg.exe -Verb RunAs -Wait -ArgumentList '{arguments}'");
        Command::new("powershell.exe")
            .args(["-NoProfile", "-NonInteractive", "-Command", &command])
            .creation_flags(Self::CREATE_NO_WINDOW)
            .status()?;
        self.is_on()
    }
}

impl Default for DeveloperMode {
    fn default() -> Self {
        Self::new()
    }
}

impl DeveloperModeControl for DeveloperMode {
    fn is_on(&self) -> Result<bool, GdkError> {
        DeveloperMode::is_on(self)
    }

    fn enable_with_consent(&self) -> Result<bool, GdkError> {
        DeveloperMode::enable_with_consent(self)
    }
}
