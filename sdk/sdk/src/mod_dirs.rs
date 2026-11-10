use std::path::PathBuf;

use crate::{Api, SdkError};

/// The folders of a mod. The loader decides where they are; none is created for the mod.
///
/// `shared_data_dir` is outside the game folder and is the same for every game instance. It is `None`
/// when the loader has no shared data root (it answers with an empty folder) or is too old to know the folder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModDirs {
    pub mod_dir: PathBuf,
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub shared_data_dir: Option<PathBuf>,
}

impl ModDirs {
    pub fn new(api: &Api, module: &str) -> Result<Self, SdkError> {
        Ok(Self {
            mod_dir: PathBuf::from(api.mod_dir(module, 0)?),
            config_dir: PathBuf::from(api.mod_dir(module, 1)?),
            data_dir: PathBuf::from(api.mod_dir(module, 2)?),
            shared_data_dir: api.mod_dir(module, 3).ok().filter(|folder| !folder.is_empty()).map(PathBuf::from),
        })
    }
}
