use super::HclText;
use crate::{LauncherError, LauncherPaths, LauncherSettings, RegistryRef};
use std::path::PathBuf;

pub struct SettingsStore {
    paths: LauncherPaths,
}

impl SettingsStore {
    const KEYS: [&'static str; 4] = ["registries", "preview", "data_dir", Self::RETIRED_DOWNLOAD_LIMIT];
    // a key an older launcher wrote; it is read without error, ignored, and left out of the next save
    const RETIRED_DOWNLOAD_LIMIT: &'static str = "download_limit_bytes";

    pub fn new(paths: LauncherPaths) -> SettingsStore {
        SettingsStore { paths }
    }

    pub fn load(&self) -> Result<LauncherSettings, LauncherError> {
        let path = self.paths.settings();
        let text = match std::fs::read_to_string(&path) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(LauncherSettings::new(LauncherSettings::normalised(Vec::new()), false, None));
            }
            Err(error) => return Err(error.into()),
        };
        let value: hcl::Value =
            hcl::from_str(&text).map_err(|error| LauncherError::BadSettings(format!("{}: {error}", path.display())))?;
        let hcl::Value::Object(map) = value else {
            return Err(LauncherError::BadSettings(format!("{} is not a settings file", path.display())));
        };
        for key in map.keys() {
            if !Self::KEYS.contains(&key.as_str()) {
                return Err(LauncherError::BadSettings(format!("unknown key {key}")));
            }
        }
        let registries = match map.get("registries") {
            None => Vec::new(),
            Some(hcl::Value::Array(items)) => {
                let mut parsed = Vec::new();
                for item in items {
                    let hcl::Value::String(raw) = item else {
                        return Err(LauncherError::BadSettings("registries must be a list of strings".to_string()));
                    };
                    parsed.push(RegistryRef::parse(raw)?);
                }
                parsed
            }
            Some(_) => return Err(LauncherError::BadSettings("registries must be a list of strings".to_string())),
        };
        let preview = match map.get("preview") {
            None => false,
            Some(hcl::Value::Bool(flag)) => *flag,
            Some(_) => return Err(LauncherError::BadSettings("preview must be true or false".to_string())),
        };
        let data_dir = match map.get("data_dir") {
            None => None,
            Some(hcl::Value::String(raw)) => Some(PathBuf::from(raw)),
            Some(_) => return Err(LauncherError::BadSettings("data_dir must be a string".to_string())),
        };
        Ok(LauncherSettings::new(LauncherSettings::normalised(registries), preview, data_dir))
    }

    pub fn save(&self, settings: &LauncherSettings) -> Result<(), LauncherError> {
        let path = self.paths.settings();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let registries = LauncherSettings::normalised(settings.registries.clone());
        let list: Vec<String> = registries.iter().map(|registry| HclText::quote(&registry.base)).collect();
        let mut text = format!("registries = [{}]\npreview    = {}\n", list.join(", "), settings.preview);
        if let Some(data_dir) = &settings.data_dir {
            text.push_str(&format!("data_dir   = {}\n", HclText::quote(&data_dir.to_string_lossy())));
        }
        let temporary = path.with_extension("hcl.tmp");
        std::fs::write(&temporary, text)?;
        std::fs::rename(&temporary, &path)?;
        Ok(())
    }
}
