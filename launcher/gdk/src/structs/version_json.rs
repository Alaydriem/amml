use crate::GdkError;
use crate::services::Timestamp;
use crate::structs::GameVersion;
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};

// An instance's `version.json`: the base keys every instance has. Every other key stays, in its original order.
#[derive(Debug, Clone)]
pub struct VersionJson {
    document: Map<String, Value>,
}

impl VersionJson {
    pub fn read(path: &Path) -> Result<VersionJson, GdkError> {
        let text = std::fs::read_to_string(path)?;
        let text = text.strip_prefix('\u{feff}').unwrap_or(&text);
        let value: Value = serde_json::from_str(text)
            .map_err(|error| GdkError::BadVersionJson(format!("{}: {error}", path.display())))?;
        match value {
            Value::Object(document) => Ok(VersionJson { document }),
            _ => Err(GdkError::BadVersionJson(format!("{} is not an object", path.display()))),
        }
    }

    // the file for a new instance: isolation off, no launch arguments, no environment
    pub fn new_for(name: &str, game: &GameVersion, preview: bool) -> VersionJson {
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
        VersionJson { document }
    }

    pub fn game_version(&self) -> Result<GameVersion, GdkError> {
        let text = self
            .document
            .get("gameVersion")
            .and_then(Value::as_str)
            .ok_or_else(|| GdkError::BadVersionJson("gameVersion is missing".to_string()))?;
        GameVersion::parse(text)
    }

    pub fn preview(&self) -> bool {
        self.document.get("type").and_then(Value::as_str).is_some_and(|kind| kind.eq_ignore_ascii_case("preview"))
    }

    pub fn value(&self, key: &str) -> Option<&Value> {
        self.document.get(key)
    }

    pub fn set(&mut self, key: &str, value: Value) {
        self.document.insert(key.to_string(), value);
    }

    // `.tmp` beside the file, then a rename; two-space indent
    pub fn write(&self, path: &Path) -> Result<(), GdkError> {
        let mut text = serde_json::to_string_pretty(&Value::Object(self.document.clone()))
            .map_err(|error| GdkError::BadVersionJson(error.to_string()))?;
        text.push('\n');
        let mut temporary = path.as_os_str().to_os_string();
        temporary.push(".tmp");
        let temporary = PathBuf::from(temporary);
        std::fs::write(&temporary, text)?;
        std::fs::rename(&temporary, path)?;
        Ok(())
    }
}
