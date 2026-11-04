use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use hcl::Expression;

use crate::SettingsError;

/// The optional per-instance file `instance.hcl` in the loader folder, written by the launcher.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InstanceSettings {
    /// The folder that stands in for the game's roaming data folder; `None` keeps `%APPDATA%`.
    pub data_dir: Option<PathBuf>,
}

impl InstanceSettings {
    /// The settings in `path`; a missing file gives the defaults. A relative `data_dir` is
    /// resolved against `game`.
    pub fn load(path: &Path, game: &Path) -> Result<Self, SettingsError> {
        Self::load_with_unknown(path, game).map(|(settings, _)| settings)
    }

    /// Like [`InstanceSettings::load`], and the attributes the loader does not know.
    pub fn load_with_unknown(path: &Path, game: &Path) -> Result<(Self, Vec<String>), SettingsError> {
        match std::fs::read_to_string(path) {
            Ok(text) => Self::parse_with_unknown(&text, game).map_err(|error| error.in_file(path)),
            Err(source) if source.kind() == ErrorKind::NotFound => Ok((Self::default(), Vec::new())),
            Err(source) => Err(SettingsError::Io { path: path.to_path_buf(), source }),
        }
    }

    /// Parses the file text. The launcher may write keys this loader does not know yet; they are
    /// returned, not refused.
    pub fn parse_with_unknown(text: &str, game: &Path) -> Result<(Self, Vec<String>), SettingsError> {
        let body = hcl::parse(text).map_err(|error| SettingsError::Syntax(error.to_string()))?;
        if body.blocks().next().is_some() {
            return Err(SettingsError::Blocks);
        }
        let mut settings = Self::default();
        let mut unknown = Vec::new();
        for attribute in body.attributes() {
            match (attribute.key(), attribute.expr()) {
                ("data_dir", Expression::String(text)) => {
                    let dir = PathBuf::from(text);
                    settings.data_dir = Some(if dir.is_absolute() { dir } else { game.join(dir) });
                }
                ("data_dir", _) => return Err(SettingsError::WrongType("data_dir".to_string())),
                (other, _) => unknown.push(other.to_string()),
            }
        }
        Ok((settings, unknown))
    }
}
