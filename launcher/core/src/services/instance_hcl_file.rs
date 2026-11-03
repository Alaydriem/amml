use super::HclText;
use crate::{DataChoice, LauncherError};
use hcl::Expression;
use std::path::{Path, PathBuf};

// An instance's `amml\instance.hcl`, which the loader reads: `data_dir` names the folder that stands in for the
// game's roaming data folder.
pub struct InstanceHclFile;

impl InstanceHclFile {
    pub fn write(path: &Path, choice: &DataChoice) -> Result<(), LauncherError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let value = match choice {
            DataChoice::Shared(dir) => dir.to_string_lossy().to_string(),
            DataChoice::Own => ".".to_string(),
        };
        let mut temporary = path.as_os_str().to_os_string();
        temporary.push(".tmp");
        let temporary = PathBuf::from(temporary);
        std::fs::write(&temporary, format!("data_dir = {}\n", HclText::quote(&value)))?;
        std::fs::rename(&temporary, path)?;
        Ok(())
    }

    pub fn exists(path: &Path) -> bool {
        path.is_file()
    }

    // `.` and any other relative path are resolved against the instance folder, as the loader does
    pub fn read_data_dir(path: &Path, instance: &Path) -> Option<PathBuf> {
        let text = std::fs::read_to_string(path).ok()?;
        let body = hcl::parse(&text).ok()?;
        for attribute in body.attributes() {
            if let ("data_dir", Expression::String(value)) = (attribute.key(), attribute.expr()) {
                let dir = PathBuf::from(value);
                return Some(if dir.is_absolute() { dir } else { instance.join(dir) });
            }
        }
        None
    }
}
