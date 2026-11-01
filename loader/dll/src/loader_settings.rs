use std::io::ErrorKind;
use std::path::Path;
use std::sync::OnceLock;

use hcl::Expression;

use crate::SettingsError;

static CURRENT: OnceLock<LoaderSettings> = OnceLock::new();

/// The optional loader config file `amml.hcl` in the loader folder.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LoaderSettings {
    /// Whether the loader grades its own addresses against LeviLamina's.
    pub grade: bool,
}

impl LoaderSettings {
    /// The settings in `path`; a missing file gives the defaults.
    pub fn load(path: &Path) -> Result<Self, SettingsError> {
        match std::fs::read_to_string(path) {
            Ok(text) => Self::parse(&text).map_err(|error| error.in_file(path)),
            Err(source) if source.kind() == ErrorKind::NotFound => Ok(Self::default()),
            Err(source) => Err(SettingsError::Io { path: path.to_path_buf(), source }),
        }
    }

    /// Parses the file text; an attribute other than `grade` is an error.
    pub fn parse(text: &str) -> Result<Self, SettingsError> {
        let body = hcl::parse(text).map_err(|error| SettingsError::Syntax(error.to_string()))?;
        if body.blocks().next().is_some() {
            return Err(SettingsError::Blocks);
        }
        let mut settings = Self::default();
        for attribute in body.attributes() {
            match (attribute.key(), attribute.expr()) {
                ("grade", Expression::Bool(value)) => settings.grade = *value,
                ("grade", _) => return Err(SettingsError::WrongType("grade".to_string())),
                (other, _) => return Err(SettingsError::UnknownAttribute(other.to_string())),
            }
        }
        Ok(settings)
    }

    /// Keeps these settings for the life of the process; the first call wins.
    pub fn publish(self) {
        let _ = CURRENT.set(self);
    }

    /// The published settings, or the defaults before `publish`.
    pub fn current() -> Self {
        CURRENT.get().copied().unwrap_or_default()
    }
}
