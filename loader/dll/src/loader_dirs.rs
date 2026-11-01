use std::path::PathBuf;

/// The folders a host gives the loader. A mod named `m` gets `mods/m` (where its DLL is),
/// `mods/m/config` and `mods/m/data`; the loader writes its own logs and each mod log under `logs`.
#[derive(Clone, Debug)]
pub struct LoaderDirs {
    pub mods: PathBuf,
    pub logs: PathBuf,
}

impl LoaderDirs {
    pub fn new(mods: PathBuf, logs: PathBuf) -> Self {
        Self { mods, logs }
    }
}
