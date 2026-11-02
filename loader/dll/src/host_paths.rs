use std::path::{Path, PathBuf};

use crate::LoaderDirs;

/// Where the loader keeps its files, derived from the game executable.
///
/// In the standalone host the loader folder is `<game>/amml/` (`recipes/`, `cache/` and the
/// optional `amml.hcl` and `instance.hcl`), logs go to `<game>/amml/logs/`, and AMML mods live in
/// `<game>/mods/amml/<name>/`.
///
/// In the LeviLamina host the loader package is `<game>/mods/amml-loader/` (the DLL, `recipes/`,
/// `cache/` and the optional `amml.hcl` and `instance.hcl`), logs go to `<game>/logs/amml/`, and AMML mods live in
/// `<game>/mods/amml/<name>/` (with `config/` and `data/` inside) so LeviLamina does not try to load
/// them itself.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostPaths {
    pub exe: PathBuf,
    pub game: PathBuf,
    pub loader: PathBuf,
    pub recipes: PathBuf,
    pub cache: PathBuf,
    pub settings: PathBuf,
    pub instance: PathBuf,
    pub logs: PathBuf,
    pub mods: PathBuf,
}

impl HostPaths {
    pub const LOADER_FOLDER: &'static str = "amml-loader";
    pub const STANDALONE_FOLDER: &'static str = "amml";
    pub const MODS_FOLDER: &'static str = "amml";
    pub const SETTINGS_FILE: &'static str = "amml.hcl";
    pub const INSTANCE_FILE: &'static str = "instance.hcl";

    /// The layout for a game executable run by LeviLamina. `<game>` is the folder of `exe_path`.
    pub fn levilamina(exe_path: &Path) -> Self {
        let game = exe_path.parent().map(Path::to_path_buf).unwrap_or_default();
        let mods_root = game.join("mods");
        let loader = mods_root.join(Self::LOADER_FOLDER);
        Self {
            exe: exe_path.to_path_buf(),
            recipes: loader.join("recipes"),
            cache: loader.join("cache"),
            settings: loader.join(Self::SETTINGS_FILE),
            instance: loader.join(Self::INSTANCE_FILE),
            logs: game.join("logs").join("amml"),
            mods: mods_root.join(Self::MODS_FOLDER),
            loader,
            game,
        }
    }

    /// The layout for a game executable that AMML starts itself. `<game>` is the folder of `exe_path`.
    pub fn standalone(exe_path: &Path) -> Self {
        let game = exe_path.parent().map(Path::to_path_buf).unwrap_or_default();
        let loader = game.join(Self::STANDALONE_FOLDER);
        Self {
            exe: exe_path.to_path_buf(),
            recipes: loader.join("recipes"),
            cache: loader.join("cache"),
            settings: loader.join(Self::SETTINGS_FILE),
            instance: loader.join(Self::INSTANCE_FILE),
            logs: loader.join("logs"),
            mods: game.join("mods").join(Self::MODS_FOLDER),
            loader,
            game,
        }
    }

    /// The folders handed to `LoaderState`.
    pub fn loader_dirs(&self) -> LoaderDirs {
        LoaderDirs::new(self.mods.clone(), self.logs.clone())
    }
}
