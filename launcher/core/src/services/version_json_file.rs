use crate::{LauncherBlock, LauncherError};
use amml_gdk::{GameVersion, GdkError, VersionJson};
use serde_json::Value;
use std::path::Path;

// An instance's `version.json` as the launcher sees it: the base keys of `amml-gdk`'s `VersionJson`, plus the
// `launcher` block.
#[derive(Debug, Clone)]
pub struct VersionJsonFile {
    document: VersionJson,
}

impl VersionJsonFile {
    const LAUNCHER_KEY: &'static str = "launcher";

    pub fn read(path: &Path) -> Result<VersionJsonFile, LauncherError> {
        VersionJson::read(path).map(|document| VersionJsonFile { document }).map_err(Self::launcher_error)
    }

    // the file for a new instance: isolation off, no launch arguments, no environment
    pub fn new_for(name: &str, game: &GameVersion, preview: bool) -> VersionJsonFile {
        VersionJsonFile { document: VersionJson::new_for(name, game, preview) }
    }

    pub fn game_version(&self) -> Result<GameVersion, LauncherError> {
        self.document.game_version().map_err(Self::launcher_error)
    }

    pub fn preview(&self) -> bool {
        self.document.preview()
    }

    pub fn launch_args(&self) -> String {
        self.text("launchArgs")
    }

    pub fn env_vars(&self) -> String {
        self.text("envVars")
    }

    pub fn isolation(&self) -> bool {
        self.document.value("enableIsolation").and_then(Value::as_bool).unwrap_or(false)
    }

    pub fn launcher(&self) -> Option<LauncherBlock> {
        self.document.value(Self::LAUNCHER_KEY).and_then(LauncherBlock::from_value)
    }

    pub fn set_isolation(&mut self, enabled: bool) {
        self.document.set("enableIsolation", Value::Bool(enabled));
    }

    pub fn set_launcher(&mut self, block: &LauncherBlock) {
        self.document.set(Self::LAUNCHER_KEY, block.to_value());
    }

    pub fn write(&self, path: &Path) -> Result<(), LauncherError> {
        self.document.write(path).map_err(Self::launcher_error)
    }

    fn text(&self, key: &str) -> String {
        self.document.value(key).and_then(Value::as_str).unwrap_or_default().to_string()
    }

    // a malformed file keeps the launcher's own error; every other cause passes through
    fn launcher_error(error: GdkError) -> LauncherError {
        match error {
            GdkError::BadVersionJson(text) => LauncherError::BadVersionJson(text),
            GdkError::Io(io) => LauncherError::from(io),
            other => LauncherError::from(other),
        }
    }
}
