use std::path::PathBuf;

use crate::{Api, SdkError};

/// The three folders of a mod. The loader decides where they are; none is created for the mod.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModDirs {
    pub mod_dir: PathBuf,
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
}

impl ModDirs {
    pub fn new(api: &Api, module: &str) -> Result<Self, SdkError> {
        Ok(Self {
            mod_dir: PathBuf::from(api.mod_dir(module, 0)?),
            config_dir: PathBuf::from(api.mod_dir(module, 1)?),
            data_dir: PathBuf::from(api.mod_dir(module, 2)?),
        })
    }
}
