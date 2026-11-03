use super::Timestamp;
use crate::{LauncherBlock, LauncherError};
use amml_gdk::GameVersion;
use serde_json::{Map, Value};
use std::path::Path;

// An instance's `version.json`. Every key the launcher does not know stays, in its original order.
#[derive(Debug, Clone)]
pub struct VersionJsonFile {
    document: Map<String, Value>,
}

impl VersionJsonFile {
    const LAUNCHER_KEY: &'static str = "launcher";

    pub fn read(path: &Path) -> Result<VersionJsonFile, LauncherError> {
        let text = std::fs::read_to_string(path)?;
        let text = text.strip_prefix('\u{feff}').unwrap_or(&text);
        let value: Value = serde_json::from_str(text)
            .map_err(|error| LauncherError::BadVersionJson(format!("{}: {error}", path.display())))?;
        match value {
            Value::Object(document) => Ok(VersionJsonFile { document }),
            _ => Err(LauncherError::BadVersionJson(format!("{} is not an object", path.display()))),
        }
    }

    // the file for a new instance: isolation off, no launch arguments, no environment
    pub fn new_for(name: &str, game: &GameVersion, preview: bool) -> VersionJsonFile {
        let mut document = Map::new();
        document.insert("name".to_string(), Value::String(name.to_string()));
        document.insert("gameVersion".to_string(), Value::String(game.instance_name()));
        document.insert("type".to_string(), Value::String(if preview { "preview" } else { "release" }.to_string()));
        document.insert("packageType".to_string(), Value::String("gdk".to_string()));
        document.insert("enableIsolation".to_string(), Value::Bool(false));
        document.insert("enableConsole".to_string(), Value::Bool(false));
        document.insert("enableEditorMode".to_string(), Value::Bool(false));
        document.insert("launchArgs".to_string(), Value::String(String::new()));
        document.insert("envVars".to_string(), Value::String(String::new()));
        document.insert("createdAt".to_string(), Value::String(Timestamp::now_rfc3339()));
        VersionJsonFile { document }
    }

    pub fn game_version(&self) -> Result<GameVersion, LauncherError> {
        let text = self
            .document
            .get("gameVersion")
            .and_then(Value::as_str)
            .ok_or_else(|| LauncherError::BadVersionJson("gameVersion is missing".to_string()))?;
        Ok(GameVersion::parse(text)?)
    }

    pub fn preview(&self) -> bool {
        self.document.get("type").and_then(Value::as_str).is_some_and(|kind| kind.eq_ignore_ascii_case("preview"))
    }

    pub fn launch_args(&self) -> String {
        self.text("launchArgs")
    }

    pub fn env_vars(&self) -> String {
        self.text("envVars")
    }

    pub fn isolation(&self) -> bool {
        self.document.get("enableIsolation").and_then(Value::as_bool).unwrap_or(false)
    }

    pub fn launcher(&self) -> Option<LauncherBlock> {
        self.document.get(Self::LAUNCHER_KEY).and_then(LauncherBlock::from_value)
    }

    pub fn set_isolation(&mut self, enabled: bool) {
        self.document.insert("enableIsolation".to_string(), Value::Bool(enabled));
    }

    pub fn set_launcher(&mut self, block: &LauncherBlock) {
        self.document.insert(Self::LAUNCHER_KEY.to_string(), block.to_value());
    }

    // `.tmp` beside the file, then a rename; two-space indent
    pub fn write(&self, path: &Path) -> Result<(), LauncherError> {
        let mut text = serde_json::to_string_pretty(&Value::Object(self.document.clone()))
            .map_err(|error| LauncherError::BadVersionJson(error.to_string()))?;
        text.push('\n');
        let mut temporary = path.as_os_str().to_os_string();
        temporary.push(".tmp");
        let temporary = std::path::PathBuf::from(temporary);
        std::fs::write(&temporary, text)?;
        std::fs::rename(&temporary, path)?;
        Ok(())
    }

    fn text(&self, key: &str) -> String {
        self.document.get(key).and_then(Value::as_str).unwrap_or_default().to_string()
    }
}
