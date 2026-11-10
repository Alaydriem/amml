use std::path::PathBuf;

/// The folders a host gives the loader. A mod named `m` gets `mods/m` (where its DLL is),
/// `mods/m/config` and `mods/m/data`; the loader writes its own logs and each mod log under `logs`.
/// The shared data folder of `m` is `mod_data/<the path of m under mods>`; with no `mod_data` a mod
/// has no shared data folder.
#[derive(Clone, Debug)]
pub struct LoaderDirs {
    pub mods: PathBuf,
    pub logs: PathBuf,
    pub mod_data: Option<PathBuf>,
}

impl LoaderDirs {
    pub fn new(mods: PathBuf, logs: PathBuf, mod_data: Option<PathBuf>) -> Self {
        Self { mods, logs, mod_data }
    }
}
